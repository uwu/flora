//! flora_macros - Procedural macros for exposing Serenity types to the Flora SDK
//!
//! This crate provides two attribute macros:
//!
//! - `#[expose_payload]` - For Rust → JS types (event payloads sent to JavaScript)
//! - `#[expose_input]` - For JS → Rust types (op arguments received from JavaScript)
//!
//! # Example
//!
//! ```rust,ignore
//! use flora_macros::{expose_payload, expose_input};
//!
//! // Event payload sent to JS
//! #[expose_payload(from = "serenity::model::user::User")]
//! pub struct UserPayload {
//!     #[expose(expr = "src.id.get().to_string()")]
//!     id: String,
//!     #[expose(expr = "src.name.clone()")]
//!     username: String,
//!     #[expose(expr = "src.bot")]
//!     bot: bool,
//! }
//!
//! // Input from JS
//! #[expose_input]
//! pub struct EmbedInput {
//!     title: Option<String>,
//!     description: Option<String>,
//! }
//! ```

mod expose_input;
mod expose_payload;

use proc_macro::TokenStream;

/// Attribute macro for types that are sent from Rust to JavaScript (event payloads).
///
/// This macro automatically adds:
/// - `#[derive(serde::Serialize, ts_rs::TS)]`
/// - `#[ts(export, export_to = "sdk/src/generated/")]`
///
/// Optionally, if `from = "path::to::Type"` is specified, it also generates
/// a `From<&SourceType>` implementation using field-level `#[expose(expr = "...")]` attributes.
///
/// # Macro Arguments
///
/// - `from = "path::to::Type"` - Generate a `From<&Type>` impl
///
/// # Field Attributes (only used when `from` is specified)
///
/// - `#[expose(expr = "...")]` - Expression to compute field value (use `src` for source)
/// - `#[expose(skip)]` - Skip this field in From impl (must have Default)
/// - `#[expose(default)]` - Use `Default::default()` for this field
///
/// # Example
///
/// ```rust,ignore
/// #[expose_payload(from = "serenity::model::user::User")]
/// pub struct UserPayload {
///     #[expose(expr = "src.id.get().to_string()")]
///     id: String,
///     #[expose(expr = "src.name.clone()")]
///     username: String,
///     #[expose(expr = "src.discriminator.map(|d| d.get())")]
///     discriminator: Option<u16>,
///     #[expose(expr = "src.bot")]
///     bot: bool,
/// }
/// ```
#[proc_macro_attribute]
pub fn expose_payload(args: TokenStream, input: TokenStream) -> TokenStream {
    expose_payload::attr_macro(args, input)
}

/// Attribute macro for types that are received from JavaScript (op arguments).
///
/// This macro automatically adds:
/// - `#[derive(Debug, serde::Deserialize, ts_rs::TS)]`
/// - `#[serde(rename_all = "camelCase")]`
/// - `#[ts(export, export_to = "sdk/src/generated/")]`
///
/// # Example
///
/// ```rust,ignore
/// #[expose_input]
/// pub struct EmbedInput {
///     title: Option<String>,
///     description: Option<String>,
/// }
/// ```
#[proc_macro_attribute]
pub fn expose_input(args: TokenStream, input: TokenStream) -> TokenStream {
    expose_input::attr_macro(args, input)
}
