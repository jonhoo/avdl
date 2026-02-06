use indexmap::IndexMap;
use serde_json::Value;

use super::schema::{AvroSchema, Field};

/// An Avro protocol.
#[derive(Debug, Clone, PartialEq)]
pub struct Protocol {
    pub name: String,
    pub namespace: Option<String>,
    pub doc: Option<String>,
    pub properties: IndexMap<String, Value>,
    pub types: Vec<AvroSchema>,
    pub messages: IndexMap<String, Message>,
}

/// An Avro protocol message (RPC method).
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub doc: Option<String>,
    pub properties: IndexMap<String, Value>,
    pub request: Vec<Field>,
    pub response: AvroSchema,
    pub errors: Option<Vec<AvroSchema>>,
    pub one_way: bool,
}
