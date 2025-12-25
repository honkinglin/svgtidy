use crate::tree::Document;

pub trait Plugin {
    fn apply(&self, doc: &mut Document);
}

pub mod cleanup_attrs;
pub mod remove_comments;
pub mod remove_doctype;
pub mod remove_editors_ns_data;
pub mod remove_empty_text;
pub mod remove_hidden_elems;
pub mod remove_metadata;
pub mod remove_xml_proc_inst;

pub use cleanup_attrs::CleanupAttrs;
pub use remove_comments::RemoveComments;
pub use remove_doctype::RemoveDoctype;
pub use remove_editors_ns_data::RemoveEditorsNSData;
pub use remove_empty_text::RemoveEmptyText;
pub use remove_hidden_elems::RemoveHiddenElems;
pub use remove_metadata::RemoveMetadata;
pub use remove_xml_proc_inst::RemoveXMLProcInst;
