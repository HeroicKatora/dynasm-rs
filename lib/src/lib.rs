// utility
extern crate lazy_static;
extern crate bitflags;
extern crate owning_ref;
extern crate byteorder;

use std::collections::HashMap;

/// Module with common infrastructure across assemblers
mod common;
/// Module with architecture-specific assembler implementations
pub mod arch;
/// Module contaning the implementation of directives
mod directive;

pub use common::{Const, Expr, Ident, JumpOffset, Size, Stmt, Value};
pub use directive::{Directive, MalformedDirectiveError};

/// output from parsing a full dynasm invocation. target represents the first dynasm argument, being the assembler
/// variable being used. stmts contains an abstract representation of the statements to be generated from this dynasm
/// invocation.
struct Dynasm {
    target: Box<dyn arch::Arch>,
    stmts: Vec<common::Stmt>
}

/// This is only compiled when the dynasm_opmap feature is used. It exports the internal assembly listings
/// into a string that can then be included into the documentation for dynasm.
#[cfg(feature = "dynasm_opmap")]
#[proc_macro]
pub fn dynasm_opmap(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {

    // parse to ensure that no macro arguments were provided
    let opmap = parse_macro_input!(tokens as DynasmOpmap);

    let mut s = String::new();
    s.push_str("% Instruction Reference\n\n");

    s.push_str(&match opmap.arch.as_str() {
        "x64" | "x86" => arch::x64::create_opmap(),
        "aarch64" => arch::aarch64::create_opmap(),
        x => panic!("Unknown architecture {}", x)
    });

    let token = quote::quote! {
        #s
    };
    token.into()
}

/// This is only compiled when the dynasm_extract feature is used. It exports the internal assembly listings
/// into a string that can then be included into the documentation for dynasm.
#[cfg(feature = "dynasm_extract")]
#[proc_macro]
pub fn dynasm_extract(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {

    // parse to ensure that no macro arguments were provided
    let opmap = parse_macro_input!(tokens as DynasmOpmap);

    let s = match opmap.arch.as_str() {
        "x64" | "x86" => "UNIMPLEMENTED".into(),
        "aarch64" => arch::aarch64::extract_opmap(),
        x => panic!("Unknown architecture {}", x)
    };

    let token = quote::quote! {
        #s
    };
    token.into()
}

/// As dynasm_opmap takes no args it doesn't parse to anything
// TODO: opmaps
struct DynasmOpmap {
    pub arch: String
}

/// This struct contains all non-parsing state that dynasm! requires while parsing and compiling
pub struct State<'a> {
    pub stmts: &'a mut Vec<common::Stmt>,
    pub target: &'a str,
    pub file_data: &'a DynasmData,
}

pub struct DynasmData {
    pub current_arch: Box<dyn arch::Arch>,
    pub aliases: HashMap<String, String>,
}

impl DynasmData {
    /// Create data with the current default architecture (target dependent).
    pub fn new() -> DynasmData {
        DynasmData {
            current_arch:
                arch::from_str(arch::CURRENT_ARCH).expect("Default architecture is invalid"),
            aliases: HashMap::new(),
        }
    }
}
