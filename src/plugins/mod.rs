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
