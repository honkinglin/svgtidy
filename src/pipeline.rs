use crate::plugins::{
    CleanupAttrs, CleanupEnableBackground, CleanupIds, CleanupListOfValues, CleanupNumericValues,
    CollapseGroups, ConvertColors, ConvertEllipseToCircle, ConvertOneStopGradients,
    ConvertPathData, ConvertShapeToPath, ConvertStyleToAttrs, ConvertTransform, InlineStyles,
    MergePaths, MergeStyles, MinifyStyles, MoveElemsAttrsToGroup, MoveGroupAttrsToElems, Plugin,
    RemoveComments, RemoveDesc, RemoveDimensions, RemoveDoctype, RemoveEditorsNSData,
    RemoveEmptyAttrs, RemoveEmptyContainers, RemoveEmptyText, RemoveHiddenElems, RemoveMetadata,
    RemoveNonInheritableGroupAttrs, RemoveRasterImages, RemoveScriptElement, RemoveStyleElement,
    RemoveTitle, RemoveUnknownsAndDefaults, RemoveUnusedNS, RemoveUselessDefs,
    RemoveUselessStrokeAndFill, RemoveXMLProcInst, SortAttrs, SortDefsChildren,
};
use crate::tree::Document;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct OptimizeOptions {
    pub precision: usize,
    pub enable: HashSet<String>,
    pub disable: HashSet<String>,
}

impl Default for OptimizeOptions {
    fn default() -> Self {
        Self {
            precision: 3,
            enable: HashSet::new(),
            disable: HashSet::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PluginDescriptor {
    pub name: &'static str,
    pub enabled_by_default: bool,
}

const PLUGIN_DESCRIPTORS: &[PluginDescriptor] = &[
    PluginDescriptor {
        name: "removeDoctype",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "removeXMLProcInst",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "removeComments",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "removeMetadata",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "removeTitle",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "removeDesc",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "removeEditorsNSData",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "removeScriptElement",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "removeRasterImages",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "removeStyleElement",
        enabled_by_default: false,
    },
    PluginDescriptor {
        name: "mergeStyles",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "minifyStyles",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "inlineStyles",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "convertStyleToAttrs",
        enabled_by_default: false,
    },
    PluginDescriptor {
        name: "cleanupAttrs",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "removeUselessStrokeAndFill",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "cleanupEnableBackground",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "removeDimensions",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "moveGroupAttrsToElems",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "moveElemsAttrsToGroup",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "convertOneStopGradients",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "cleanupIds",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "removeUselessDefs",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "removeEmptyContainers",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "removeHiddenElems",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "removeEmptyText",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "collapseGroups",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "convertEllipseToCircle",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "convertShapeToPath",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "convertPathData",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "convertTransform",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "cleanupNumericValues",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "cleanupListOfValues",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "removeUnknownsAndDefaults",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "removeNonInheritableGroupAttrs",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "mergePaths",
        enabled_by_default: false,
    },
    PluginDescriptor {
        name: "convertColors",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "removeEmptyAttrs",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "removeUnusedNS",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "sortAttrs",
        enabled_by_default: true,
    },
    PluginDescriptor {
        name: "sortDefsChildren",
        enabled_by_default: true,
    },
];

pub fn plugin_descriptors() -> &'static [PluginDescriptor] {
    PLUGIN_DESCRIPTORS
}

pub fn apply_default_pipeline(doc: &mut Document, options: &OptimizeOptions) {
    let features = document_features(doc);
    for plugin in resolve_plugins(options, &features) {
        plugin.apply(doc);
    }
}

pub fn unknown_plugin_names(options: &OptimizeOptions) -> Vec<String> {
    let known: HashSet<&str> = plugin_descriptors()
        .iter()
        .map(|plugin| plugin.name)
        .collect();
    let mut unknown: Vec<String> = options
        .enable
        .iter()
        .chain(options.disable.iter())
        .filter(|name| !known.contains(name.as_str()))
        .cloned()
        .collect();
    unknown.sort();
    unknown.dedup();
    unknown
}

fn resolve_plugins(options: &OptimizeOptions, features: &DocumentFeatures) -> Vec<Box<dyn Plugin>> {
    plugin_descriptors()
        .iter()
        .filter(|descriptor| is_enabled(descriptor, options))
        .filter(|descriptor| should_run_plugin(descriptor.name, features))
        .map(|descriptor| build_plugin(descriptor.name, options.precision))
        .collect()
}

#[derive(Default)]
struct DocumentFeatures {
    has_group: bool,
    has_style_element: bool,
    has_style_attr: bool,
    has_enable_background: bool,
}

fn document_features(doc: &Document) -> DocumentFeatures {
    let mut features = DocumentFeatures::default();
    collect_document_features(&doc.root, &mut features);
    features
}

fn collect_document_features(nodes: &[crate::tree::Node], features: &mut DocumentFeatures) {
    for node in nodes {
        if let crate::tree::Node::Element(elem) = node {
            if elem.name == "g" {
                features.has_group = true;
            }
            if elem.name == "style" {
                features.has_style_element = true;
            }
            if elem.attributes.contains_key("style") {
                features.has_style_attr = true;
            }
            if elem.attributes.contains_key("enable-background") {
                features.has_enable_background = true;
            }
            collect_document_features(&elem.children, features);
        }
    }
}

fn should_run_plugin(name: &str, features: &DocumentFeatures) -> bool {
    match name {
        "mergeStyles" => features.has_style_element,
        "inlineStyles" => features.has_style_element,
        "minifyStyles" => features.has_style_element || features.has_style_attr,
        "convertStyleToAttrs" => features.has_style_attr,
        "removeStyleElement" => features.has_style_element,
        "cleanupEnableBackground" => features.has_enable_background || features.has_style_attr,
        "moveGroupAttrsToElems"
        | "moveElemsAttrsToGroup"
        | "removeNonInheritableGroupAttrs"
        | "collapseGroups" => features.has_group,
        _ => true,
    }
}

fn is_enabled(descriptor: &PluginDescriptor, options: &OptimizeOptions) -> bool {
    if options.disable.contains(descriptor.name) {
        return false;
    }

    if options.enable.contains(descriptor.name) {
        return true;
    }

    descriptor.enabled_by_default
}

fn build_plugin(name: &str, precision: usize) -> Box<dyn Plugin> {
    match name {
        "removeDoctype" => Box::new(RemoveDoctype),
        "removeXMLProcInst" => Box::new(RemoveXMLProcInst),
        "removeComments" => Box::new(RemoveComments),
        "removeMetadata" => Box::new(RemoveMetadata),
        "removeTitle" => Box::new(RemoveTitle),
        "removeDesc" => Box::new(RemoveDesc),
        "removeEditorsNSData" => Box::new(RemoveEditorsNSData),
        "removeScriptElement" => Box::new(RemoveScriptElement),
        "removeRasterImages" => Box::new(RemoveRasterImages),
        "removeStyleElement" => Box::new(RemoveStyleElement),
        "mergeStyles" => Box::new(MergeStyles),
        "inlineStyles" => Box::new(InlineStyles),
        "minifyStyles" => Box::new(MinifyStyles),
        "convertStyleToAttrs" => Box::new(ConvertStyleToAttrs),
        "cleanupAttrs" => Box::new(CleanupAttrs),
        "removeUselessStrokeAndFill" => Box::new(RemoveUselessStrokeAndFill),
        "cleanupEnableBackground" => Box::new(CleanupEnableBackground),
        "removeDimensions" => Box::new(RemoveDimensions),
        "moveGroupAttrsToElems" => Box::new(MoveGroupAttrsToElems),
        "moveElemsAttrsToGroup" => Box::new(MoveElemsAttrsToGroup),
        "convertOneStopGradients" => Box::new(ConvertOneStopGradients),
        "cleanupIds" => Box::new(CleanupIds),
        "removeUselessDefs" => Box::new(RemoveUselessDefs),
        "removeEmptyContainers" => Box::new(RemoveEmptyContainers),
        "removeHiddenElems" => Box::new(RemoveHiddenElems),
        "removeEmptyText" => Box::new(RemoveEmptyText),
        "collapseGroups" => Box::new(CollapseGroups),
        "convertEllipseToCircle" => Box::new(ConvertEllipseToCircle),
        "convertShapeToPath" => Box::new(ConvertShapeToPath),
        "convertPathData" => Box::new(ConvertPathData {
            float_precision: precision,
            leading_zero: true,
            ..Default::default()
        }),
        "convertTransform" => Box::new(ConvertTransform {
            float_precision: precision,
            deg_precision: precision,
            ..Default::default()
        }),
        "cleanupNumericValues" => Box::new(CleanupNumericValues {
            float_precision: precision,
            remove_px: true,
            leading_zero: true,
        }),
        "cleanupListOfValues" => Box::new(CleanupListOfValues {
            float_precision: precision,
            default_px: true,
            convert_to_px: true,
            leading_zero: true,
        }),
        "removeUnknownsAndDefaults" => Box::new(RemoveUnknownsAndDefaults::default()),
        "removeNonInheritableGroupAttrs" => Box::new(RemoveNonInheritableGroupAttrs),
        "mergePaths" => Box::new(MergePaths),
        "convertColors" => Box::new(ConvertColors),
        "removeEmptyAttrs" => Box::new(RemoveEmptyAttrs),
        "removeUnusedNS" => Box::new(RemoveUnusedNS),
        "sortAttrs" => Box::new(SortAttrs),
        "sortDefsChildren" => Box::new(SortDefsChildren),
        _ => unreachable!("unknown plugin: {name}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::printer;

    #[test]
    fn test_unknown_plugin_names() {
        let mut options = OptimizeOptions::default();
        options.enable.insert("missingA".to_string());
        options.disable.insert("missingB".to_string());
        options.disable.insert("missingA".to_string());

        assert_eq!(
            unknown_plugin_names(&options),
            vec!["missingA".to_string(), "missingB".to_string()]
        );
    }

    #[test]
    fn test_merge_paths_disabled_by_default() {
        let descriptor = plugin_descriptors()
            .iter()
            .find(|descriptor| descriptor.name == "mergePaths")
            .unwrap();

        assert!(!descriptor.enabled_by_default);
    }

    #[test]
    fn test_convert_style_to_attrs_disabled_by_default() {
        let descriptor = plugin_descriptors()
            .iter()
            .find(|descriptor| descriptor.name == "convertStyleToAttrs")
            .unwrap();

        assert!(!descriptor.enabled_by_default);
    }

    #[test]
    fn test_style_pipeline_runs_when_style_is_present() {
        let mut doc =
            parser::parse("<svg><style>.a { fill: red; }</style><rect class=\"a\"/></svg>")
                .unwrap();

        apply_default_pipeline(&mut doc, &OptimizeOptions::default());

        assert_eq!(printer::print(&doc), "<svg><rect style=\"fill:red\"/></svg>");
    }

    #[test]
    fn test_style_pipeline_inlines_after_minifying_stylesheet() {
        let mut doc = parser::parse(
            "<svg><style>/* comment */ .a { fill : red ; stroke : blue ; }</style><path class=\"a\" d=\"M0 0\"/></svg>",
        )
        .unwrap();

        apply_default_pipeline(&mut doc, &OptimizeOptions::default());

        assert_eq!(
            printer::print(&doc),
            "<svg><path d=\"M0 0\" style=\"fill:red;stroke:#00f\"/></svg>"
        );
    }
}
