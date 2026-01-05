use crate::tree::Document;

pub trait Plugin {
    fn apply(&self, doc: &mut Document);
}

pub mod cleanup_attrs;
pub mod convert_colors;
pub mod remove_comments;
pub mod remove_doctype;
pub mod remove_editors_ns_data;
pub mod remove_empty_text;
pub mod remove_hidden_elems;
pub mod remove_metadata;
pub mod remove_xml_proc_inst;

pub mod cleanup_ids;
pub mod cleanup_numeric_values;
pub mod collapse_groups;
pub mod collections;
pub mod remove_useless_defs;

pub mod convert_path_data;
pub mod convert_shape_to_path;
pub mod remove_desc;
pub mod remove_empty_attrs;
pub mod remove_title;

pub mod convert_style_to_attrs;
pub mod convert_transform;
pub mod merge_paths;

pub mod cleanup_list_of_values;
pub mod remove_unknowns_and_defaults;
pub mod sort_attrs;

pub mod remove_dimensions;
pub mod remove_raster_images;
pub mod remove_script_element;

pub mod convert_ellipse_to_circle;
pub mod remove_style_element;
pub mod remove_useless_stroke_and_fill;

pub mod convert_one_stop_gradients;
pub mod move_elems_attrs_to_group;
pub mod move_group_attrs_to_elems;

pub use cleanup_attrs::CleanupAttrs;
pub use convert_colors::ConvertColors;
pub use remove_comments::RemoveComments;
pub use remove_doctype::RemoveDoctype;
pub use remove_editors_ns_data::RemoveEditorsNSData;
pub use remove_empty_text::RemoveEmptyText;
pub use remove_hidden_elems::RemoveHiddenElems;
pub use remove_metadata::RemoveMetadata;
pub use remove_xml_proc_inst::RemoveXMLProcInst;

pub use cleanup_ids::CleanupIds;
pub use cleanup_numeric_values::CleanupNumericValues;
pub use collapse_groups::CollapseGroups;
pub use remove_useless_defs::RemoveUselessDefs;

pub use convert_path_data::ConvertPathData;
pub use convert_shape_to_path::ConvertShapeToPath;
pub use remove_desc::RemoveDesc;
pub use remove_empty_attrs::RemoveEmptyAttrs;
pub use remove_title::RemoveTitle;

pub use convert_style_to_attrs::ConvertStyleToAttrs;
pub use convert_transform::ConvertTransform;
pub use merge_paths::MergePaths;

pub use cleanup_list_of_values::CleanupListOfValues;
pub use remove_unknowns_and_defaults::RemoveUnknownsAndDefaults;
pub use sort_attrs::SortAttrs;

pub use remove_dimensions::RemoveDimensions;
pub use remove_raster_images::RemoveRasterImages;
pub use remove_script_element::RemoveScriptElement;

pub use convert_ellipse_to_circle::ConvertEllipseToCircle;
pub use remove_style_element::RemoveStyleElement;
pub use remove_useless_stroke_and_fill::RemoveUselessStrokeAndFill;

pub use convert_one_stop_gradients::ConvertOneStopGradients;
pub use move_elems_attrs_to_group::MoveElemsAttrsToGroup;
pub use move_group_attrs_to_elems::MoveGroupAttrsToElems;
