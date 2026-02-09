pub(crate) mod generated;

pub(crate) mod compiler;
pub(crate) mod doc_comments;
pub(crate) mod error;
pub(crate) mod import;
pub(crate) mod model;
pub(crate) mod reader;
pub(crate) mod resolve;

// Re-export the small number of public API at the crate root.
pub use compiler::{Idl, Idl2Schemata, IdlOutput, NamedSchema, SchemataOutput, to_json_string};
