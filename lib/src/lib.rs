// utility
extern crate lazy_static;
extern crate bitflags;
extern crate byteorder;

use std::collections::HashMap;

/// Module with common infrastructure across assemblers
pub mod common;
/// Module with architecture-specific assembler implementations
pub mod arch;
/// Module containing the implementation of directives
mod directive;

pub use common::{Const, Expr, Ident, Number, NumericRepr, JumpOffset, Size, Stmt, Value};
pub use directive::{Directive, MalformedDirectiveError};

/// An assembler that simply collects all statements in order.
///
/// This makes it possible to replay the assembly process if no external expressions had to be
/// resolved into a more permanent or lower representation of the machine code.
///
/// A higher level wrapper can also defer to it for basic operations and only implement some logic
/// for expression resolving, diagnostics, etc. on top.
pub struct BasicAssembler {
    /// All collected statements in their order.
    pub stmts: Vec<common::Stmt>,
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
