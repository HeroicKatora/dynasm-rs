use crate::State;
use crate::common::{Expr, Jump, Size, Stmt, Value};

use std::fmt::{self, Debug};

pub mod x64;
// pub mod aarch64;

pub trait Arch: Debug + Send {
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
    /// Append a new statement.
    fn push(&mut self, _: Stmt);
    /// a | b
    fn bit_or(&mut self, _: Expr, _: Value) -> Option<Expr>;
    /// a & b
    fn bit_and(&mut self, _: Expr, _: Value) -> Option<Expr>;
    /// a ^ b
    fn bit_xor(&mut self, _: Expr, _: Value) -> Option<Expr>;
    /// a + b
    fn add(&mut self, _: Expr, _: Value) -> Option<Expr>;
    /// a * b
    fn mul(&mut self, _: Expr, _: Value) -> Option<Expr>;
    /// !a
    fn neg(&mut self, _: Expr) -> Option<Expr>;
    /// Log2, mostly used to encode scalings.
    fn log2(&mut self, _: Expr) -> Option<Expr>;
    /// (val & mask) << shift
    fn mask_shift(&mut self, val: Expr, mask: u64, shift: i8) -> Option<Expr>;
    /// Emit an error message.
    /// When any error is generated then the instruction compilation is expected to fail.
    fn emit_error_at(&mut self, _: ErrorSpan, _: fmt::Arguments);
}

#[derive(Debug, Clone)]
pub enum Error {
    /// Expressions had to be combined but that failed.
    BadExprCombinator {
        /// The expression that should have been added to some other value.
        expr: Expr,
    },
}

/// An opaque description of an error origin.
#[derive(Debug, Clone, Copy)]
pub enum ErrorSpan {
    InstructionPart {
        idx: usize,
    },
    Argument {
        idx: usize,
    },
}

impl ErrorSpan {
    pub fn instruction_part(idx: usize) -> Self {
        ErrorSpan::InstructionPart { idx }
    }

    pub fn argument(idx: usize) -> Self {
        ErrorSpan::Argument { idx }
    }
}

pub trait BasicExprBuilderExt: BasicExprBuilder {
    fn bit_or_else_err(&mut self, a: Expr, b: Value) -> Result<Expr, Error> {
        self.bit_or(a, b).ok_or_else(|| Error::BadExprCombinator { expr: a })
    }

    fn bit_and_else_err(&mut self, a: Expr, b: Value) -> Result<Expr, Error> {
        self.bit_and(a, b).ok_or_else(|| Error::BadExprCombinator { expr: a })
    }

    fn bit_xor_else_err(&mut self, a: Expr, b: Value) -> Result<Expr, Error> {
        self.bit_xor(a, b).ok_or_else(|| Error::BadExprCombinator { expr: a })
    }

    fn neg_else_err(&mut self, a: Expr) -> Result<Expr, Error> {
        self.neg(a).ok_or_else(|| Error::BadExprCombinator { expr: a })
    }

    fn add_else_err(&mut self, a: Expr, b: Value) -> Result<Expr, Error> {
        self.add(a, b).ok_or_else(|| Error::BadExprCombinator { expr: a })
    }

    fn mul_else_err(&mut self, a: Expr, b: Value) -> Result<Expr, Error> {
        self.mul(a, b).ok_or_else(|| Error::BadExprCombinator { expr: a })
    }

    fn add_many(&mut self, iter: impl IntoIterator<Item=Expr>) -> Option<Value> {
        let mut res = Value::Byte(0);
        for expr in iter {
            res = self.add(expr, res)?.into();
        }
        Some(res)
    }

    fn add_many_else_err(&mut self, iter: impl IntoIterator<Item=Expr>) -> Result<Value, Error> {
        let mut res = Value::Byte(0);
        for expr in iter {
            res = self.add_else_err(expr, res)?.into();
        }
        Ok(res)
    }

    /// reg | ((val & mask) << shift)
    /// `val` is not a Value since then the lib could already constant fold until `|`.
    fn mask_shift_or(&mut self, reg: Value, val: Expr, mask: u64, shift: i8) -> Option<Expr> {
        let operand = self.mask_shift(val, mask, shift)?;
        self.bit_or(operand, reg)
    }

    /// reg | ((val & mask) << shift)
    fn mask_shift_or_else_err(&mut self, reg: Value, val: Expr, mask: u64, shift: i8) -> Result<Expr, Error> {
        let operand = self.mask_shift_else_err(val, mask, shift)?;
        self.bit_or_else_err(operand, reg)
    }

    /// reg & !((val & mask) << shift)
    /// `val` is not a Value since then the lib could already constant fold until `|`.
    fn mask_shift_inverted_and(&mut self, reg: Value, val: Expr, mask: u64, shift: i8) -> Option<Expr> {
        let operand = self.mask_shift(val, mask, shift)?;
        let operand = self.neg(operand)?;
        self.bit_and(operand, reg)
    }

    /// reg | ((val & mask) << shift)
    fn mask_shift_inverted_and_else_err(&mut self, reg: Value, val: Expr, mask: u64, shift: i8) -> Result<Expr, Error> {
        let operand = self.mask_shift_else_err(val, mask, shift)?;
        let operand = self.neg_else_err(operand)?;
        self.bit_and_else_err(operand, reg)
    }

    fn log2_else_err(&mut self, a: Expr) -> Result<Expr, Error> {
        self.log2(a).ok_or_else(|| Error::BadExprCombinator { expr: a })
    }

    fn dynscale(&mut self, _: Expr, _: Value) -> Result<(Expr, Expr), Error> {
        unimplemented!()
    }

    fn mask_shift_else_err(&mut self, val: Expr, mask: u64, shift: i8) -> Result<Expr, Error> {
        self.mask_shift(val, mask, shift).ok_or_else(|| Error::BadExprCombinator { expr: val })
    }
}

impl<T: BasicExprBuilder + ?Sized> BasicExprBuilderExt for T { }

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

/// A simple implementation of a `BasicExprBuilder`.
///
/// It can not combine any expressions, pushes statements into a `Vec` and emits errors onto
/// standard error directly.
impl BasicExprBuilder for State<'_> {
    fn push(&mut self, stmt: Stmt) {
        self.stmts.push(stmt)
    }

    /// Emits the error message on `stderr`.
    fn emit_error_at(&mut self, _: ErrorSpan, args: fmt::Arguments) {
        eprintln!("{}", args);
    }

    fn bit_or(&mut self, _: Expr, _: Value) -> Option<Expr> {
        None 
    }

    fn bit_and(&mut self, _: Expr, _: Value) -> Option<Expr> {
        None
    }

    fn bit_xor(&mut self, _: Expr, _: Value) -> Option<Expr> {
        None
    }

    fn add(&mut self, _: Expr, _: Value) -> Option<Expr> {
        None
    }

    fn mul(&mut self, _: Expr, _: Value) -> Option<Expr> {
        None
    }

    fn neg(&mut self, _: Expr) -> Option<Expr> {
        None
    }

    fn log2(&mut self, _: Expr) -> Option<Expr> {
        None
    }

    fn mask_shift(&mut self, _: Expr, _: u64, _: i8) -> Option<Expr> {
        None
    }
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
