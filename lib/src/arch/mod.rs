use crate::common::{Expr, Jump, Size, Stmt, Value};

use std::fmt::Debug;

pub mod x64;
// pub mod aarch64;

pub(crate) trait Arch: Debug + Send {
    fn name(&self) -> &str;
    fn set_features(&mut self, features: &[String]);
    fn handle_static_reloc(&self, stmts: &mut Vec<Stmt>, reloc: Jump, size: Size);
    fn default_align(&self) -> u8;
}

/// An environment that can dynamically build expressions from values.
///
/// These are used to integrate dynamic user expression into assembled code where values appear
/// only in a modified form.
pub trait BasicExprBuilder {
    /// a | b
    fn bit_or(&mut self, _: Expr, _: u64) -> Option<Expr>;
    /// a & b
    fn bit_and(&mut self, _: Expr, _: u64) -> Option<Expr>;
    /// a ^ b
    fn bit_xor(&mut self, _: Expr, _: u64) -> Option<Expr>;
    /// !a
    fn neg(&mut self, _: Expr) -> Option<Expr>;
    /// Log2, mostly used to encode scalings.
    fn log2(&mut self, _: Expr) -> Option<Expr>;
    /// len*size
    fn scaled(&mut self, len: Value, size: Value) -> Option<Expr>;
    /// (v & mask) << shift
    fn mask_shift(&mut self, _: Value, mask: u64, shift: i8) -> Option<Expr>;
}

pub enum Error {
    BadArgument {
        message: String,
    },
}

impl Error {
    fn emit_error_at(message: String) -> Self {
        Error::BadArgument { message }
    }
}

/// A `BasicExprBuilder` that is not capable of building any of the expressions.
pub struct NoExpressionCombinators;

#[derive(Clone, Debug)]
pub struct DummyArch {
    name: &'static str
}

impl DummyArch {
    fn new(name: &'static str) -> DummyArch {
        DummyArch { name }
    }
}

impl Arch for DummyArch {
    fn name(&self) -> &str {
        self.name
    }

    fn set_features(&mut self, features: &[String]) {
        if let Some(feature) = features.first() {
            eprintln!("Cannot set features when the assembling architecture is undefined. Define it using a .arch directive");
        }
    }

    fn handle_static_reloc(&self, _stmts: &mut Vec<Stmt>, _reloc: Jump, _size: Size) {
        eprintln!("Current assembling architecture is undefined. Define it using a .arch directive");
    }

    fn default_align(&self) -> u8 {
        0
    }
}

impl BasicExprBuilder for NoExpressionCombinators {
    fn bit_or(&mut self, _: Expr, _: u64) -> Option<Expr> { None }
    fn bit_and(&mut self, _: Expr, _: u64) -> Option<Expr> { None }
    fn bit_xor(&mut self, _: Expr, _: u64) -> Option<Expr> { None }
    fn neg(&mut self, _: Expr) -> Option<Expr> { None }
    fn log2(&mut self, _: Expr) -> Option<Expr> { None }
    fn scaled(&mut self, _: Value, _: Value) -> Option<Expr> { None }
    fn mask_shift(&mut self, _: Value, _: u64, _: i8) -> Option<Expr> { None }
}

pub(crate) fn from_str(s: &str) -> Option<Box<dyn Arch>> {
    match s {
        "x64" => Some(Box::new(x64::Archx64::default())),
        "x86" => Some(Box::new(x64::Archx86::default())),
        // "aarch64" => Some(Box::new(aarch64::ArchAarch64::default())),
        "unknown" => Some(Box::new(DummyArch::new("unknown"))),
        _ => None
    }
}

#[cfg(target_arch="x86_64")]
pub const CURRENT_ARCH: &str = "x64";
#[cfg(target_arch="x86")]
pub const CURRENT_ARCH: &str = "x86";
#[cfg(target_arch="aarch64")]
pub const CURRENT_ARCH: &str = "aarch64";
#[cfg(not(any(target_arch="x86", target_arch="x86_64", target_arch="aarch64")))]
pub const CURRENT_ARCH: &str = "unknown";
