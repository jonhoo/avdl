//! Avro IDL compiler — parse `.avdl` files and emit Avro protocol (`.avpr`) or
//! schema (`.avsc`) JSON.
//!
//! This crate provides two main entry points, mirroring the `avro-tools` CLI
//! subcommands:
//!
//! - [`Idl`] — compile a `.avdl` file to a single JSON value (protocol or
//!   schema). Equivalent to `avro-tools idl`.
//! - [`Idl2Schemata`] — extract individual named schemas from a `.avdl` file,
//!   each as a self-contained `.avsc` JSON value. Equivalent to
//!   `avro-tools idl2schemata`.
//!
//! Both are non-consuming builders that can be reused across multiple calls.
//!
//! # Compiling a protocol
//!
//! ```no_run
//! use avdl::Idl;
//!
//! let output = Idl::new()
//!     .import_dir("schemas/shared/")
//!     .convert("schemas/service.avdl")?;
//! println!("{}", serde_json::to_string_pretty(&output.json)?);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Extracting individual schemas
//!
//! ```no_run
//! use avdl::Idl2Schemata;
//!
//! let output = Idl2Schemata::new()
//!     .extract("schemas/service.avdl")?;
//! for schema in &output.schemas {
//!     std::fs::write(
//!         format!("{}.avsc", schema.name),
//!         serde_json::to_string_pretty(&schema.schema)?,
//!     )?;
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Error handling
//!
//! All fallible methods return [`miette::Result`], which provides rich
//! diagnostic output with source spans when printed with `{:?}`.

pub(crate) mod generated;

pub(crate) mod compiler;
pub(crate) mod doc_comments;
pub(crate) mod error;
pub(crate) mod import;
pub(crate) mod model;
pub(crate) mod reader;
pub(crate) mod resolve;

// Re-export the small number of public API at the crate root.
pub use compiler::{Idl, Idl2Schemata, IdlOutput, NamedSchema, SchemataOutput};
