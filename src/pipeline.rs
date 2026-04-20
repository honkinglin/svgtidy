use crate::plugins::{
    CleanupAttrs, CleanupIds, CleanupListOfValues, CleanupNumericValues, CollapseGroups,
    ConvertColors, ConvertEllipseToCircle, ConvertOneStopGradients, ConvertPathData,
    ConvertShapeToPath, ConvertStyleToAttrs, ConvertTransform, MergePaths, MoveElemsAttrsToGroup,
    MoveGroupAttrsToElems, Plugin, RemoveComments, RemoveDesc, RemoveDimensions, RemoveDoctype,
    RemoveEditorsNSData, RemoveEmptyAttrs, RemoveEmptyContainers, RemoveEmptyText,
    RemoveHiddenElems, RemoveMetadata, RemoveRasterImages, RemoveScriptElement, RemoveStyleElement,
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
        name: "convertStyleToAttrs",
        enabled_by_default: true,
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
    for plugin in resolve_plugins(options) {
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

fn resolve_plugins(options: &OptimizeOptions) -> Vec<Box<dyn Plugin>> {
    plugin_descriptors()
        .iter()
        .filter(|descriptor| is_enabled(descriptor, options))
        .map(|descriptor| build_plugin(descriptor.name, options.precision))
        .collect()
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
        "convertStyleToAttrs" => Box::new(ConvertStyleToAttrs),
        "cleanupAttrs" => Box::new(CleanupAttrs),
        "removeUselessStrokeAndFill" => Box::new(RemoveUselessStrokeAndFill),
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
}
