//! This is a support crate that contains the function-like procedural macro to
//! build state machines using the [sm] crate. All documentation lives in that
//! crate.
//!
//! [sm]: https://docs.rs/sm

// quote! macro needs a higher recursion limit
#![recursion_limit = "512"]
#![forbid(
    future_incompatible,
    macro_use_extern_crate,
    missing_copy_implementations,
    missing_debug_implementations,
    nonstandard_style,
    rust_2018_compatibility,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    variant_size_differences
)]
#![warn(
    missing_docs,
    non_snake_case,
    rust_2018_idioms,
    single_use_lifetimes,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    unused
)]
#![deny(clippy::all)]

extern crate proc_macro;

use crate::fsm::machine::Machine;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

mod error;
mod fsm;

/// Generate the declaratively described state machine diagram.
///
/// See the main crate documentation for more details.
#[proc_macro]
pub fn fsm(input: TokenStream) -> TokenStream {
    let machines: Machine = parse_macro_input!(input as Machine);

    quote!(#machines).into()
}
