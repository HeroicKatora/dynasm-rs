// utility
extern crate lazy_static;
extern crate bitflags;
extern crate byteorder;

use std::collections::HashMap;

/// Module with common infrastructure across assemblers
mod common;
/// Module with architecture-specific assembler implementations
pub mod arch;
/// Module contaning the implementation of directives
mod directive;

pub use common::{Const, Expr, Ident, Number, NumericRepr, JumpOffset, Size, Stmt, Value};
pub use directive::{Directive, MalformedDirectiveError};

/// output from parsing a full dynasm invocation. target represents the first dynasm argument, being the assembler
/// variable being used. stmts contains an abstract representation of the statements to be generated from this dynasm
/// invocation.
struct Dynasm {
    target: Box<dyn arch::Arch>,
    stmts: Vec<common::Stmt>
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
    pub file_data: &'a mut DynasmData,
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
