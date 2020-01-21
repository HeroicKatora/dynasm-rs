pub mod ast;
mod compiler;
pub mod parser;
mod debug;
mod x64data;

use std::borrow::Cow;

use crate::State;
use crate::arch::{Arch, Error as ExprBuilderError, BasicExprBuilder};
use crate::common::{Size, Stmt, Jump};

#[cfg(feature = "dynasm_opmap")]
pub use debug::create_opmap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum X86Mode {
    Long,
    Protected
}

struct Context<'a> {
    pub state: &'a mut dyn BasicExprBuilder,
    pub mode: X86Mode,
    pub features: x64data::Features
}

#[derive(Clone, Debug)]
pub struct Archx64 {
    features: x64data::Features,
}

#[derive(Clone, Debug)]
pub struct Archx86 {
    features: x64data::Features,
}

#[derive(Debug)]
pub struct InstructionX64 {
    pub inst: ast::Instruction,
    pub args: Vec<ast::CleanArg>,
}

#[derive(Debug)]
pub struct InstructionX86 {
    pub inst: ast::Instruction,
    pub args: Vec<ast::CleanArg>,
}

impl Default for Archx64 {
    fn default() -> Archx64 {
        Archx64 { features: x64data::Features::all() }
    }
}

impl Default for Archx86 {
    fn default() -> Archx86 {
        Archx86 { features: x64data::Features::all() }
    }
}

pub trait AssembleX64 {
    /// Turn an expression into binary format.
    /// May error when dynamic data is present at bad locations such as memory address scaling.
    fn compile_instruction(&mut self, arch: &Archx64, _: InstructionX64) -> Result<(), Error>;

    /// Create an instruction composed from dynamic data.
    /// Only available when the type is also capable of building new composite expressions.
    fn build_instruction(&mut self, arch: &Archx64, _: InstructionX64) -> Result<(), Error>
        where Self: BasicExprBuilder;
}

pub trait AssembleX86 {
    /// Turn an expression into binary format.
    /// May error when dynamic data is present at bad locations such as memory address scaling.
    fn compile_instruction(&mut self, arch: &Archx86, _: InstructionX86) -> Result<(), Error>;

    /// Create an instruction composed from dynamic data.
    /// Only available when the type is also capable of building new composite expressions.
    fn build_instruction(&mut self, arch: &Archx86, _: InstructionX86) -> Result<(), Error>
        where Self: BasicExprBuilder;
}

/// An error while assembling, either an error in the environment or during processing.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Error {
    /// An error happened within the expression builder.
    Expr(ExprBuilderError),
    /// Use of features that were disabled or would need to be explicitly enabled.
    // format!( "This instruction uses features that are not indicated to be available: {}", data.features - ctx.features).into());
    DisabledFeatures(x64data::Features),
    /// An operand would be supported in another mode but not this one.
    /// This may be due to being unimplemented.
    UnsupportedOperandInThisMode {
        /// The stringified operand.
        operand: String,
        /// The size of the operand.
        op_size: Size,
        /// The mode where the error happened.
        mode: X86Mode,
        /// The mode where it would be supported.
        mode_hint: Option<X86Mode>,
    },
    /// A more generic version of the previous.
    /// Not all messages have been implemented in full detail.
    UnsupportedInThisMode {
        message: Cow<'static, str>,
        mode_hint: Option<X86Mode>,
    },
    /// An error without occurred where diagnostics offer no introspection.
    /// This should be slowly phased out. Hint: Add a `#[deprecated]` to this variant to show
    /// remaining instances.
    Generic {
        message: Cow<'static, str>,
    },
    /// Some unspecified consistency check did not succeed.
    /// When this occurs we have emitted one or several diagnostic messages.
    Fatal,
}

impl From<ExprBuilderError> for Error {
    fn from(err: ExprBuilderError) -> Self {
        Error::Expr(err)
    }
}

impl From<&'static str> for Error {
    fn from(message: &'static str) -> Self {
        Error::Generic {
            message: Cow::Borrowed(message),
        }
    }
}

impl From<String> for Error {
    fn from(message: String) -> Self {
        Error::Generic {
            message: Cow::Owned(message),
        }
    }
}

impl Arch for Archx64 {
    fn name(&self) -> &str {
        "x64"
    }

    fn set_features(&mut self, features: &[String]) {
        let mut new_features = x64data::Features::empty();
        for ident in features {
            new_features |= match x64data::Features::from_str(&ident.to_string()) {
                Some(feature) => feature,
                None => {
                    eprintln!("Architecture x64 does not support feature '{}'", ident.to_string());
                    continue;
                }
            }
        }
        self.features = new_features;
    }

    fn handle_static_reloc(&self, stmts: &mut Vec<Stmt>, reloc: Jump, size: Size) {
        let data = [0, size.in_bytes()]; // no offset, specified size, relative implicit

        stmts.push(Stmt::zeroed(size));
        stmts.push(reloc.encode(&data));
    }

    fn default_align(&self) -> u8 {
        0x90
    }
}

impl AssembleX64 for State<'_> {
    fn compile_instruction(&mut self, arch: &Archx64, instruction: InstructionX64) -> Result<(), Error> {
        let InstructionX64 { inst, args } = instruction;

        let ctx = Context {
            state: self,
            mode: X86Mode::Long,
            features: arch.features,
        };

        compiler::compile_instruction(ctx, inst, args)
    }

    fn build_instruction(&mut self, _: &Archx64, _: InstructionX64) -> Result<(), Error>
        where Self: BasicExprBuilder
    {
        unreachable!("Statically uncallable, Self is not BasicExprBuilder")
    }
}

impl Arch for Archx86 {
    fn name(&self) -> &str {
        "x86"
    }

    fn set_features(&mut self, features: &[String]) {
        let mut new_features = x64data::Features::empty();
        for ident in features {
            new_features |= match x64data::Features::from_str(&ident.to_string()) {
                Some(feature) => feature,
                None => {
                    eprintln!("Architecture x86 does not support feature '{}'", ident.to_string());
                    continue;
                }
            }
        }
        self.features = new_features;
    }

    fn handle_static_reloc(&self, stmts: &mut Vec<Stmt>, reloc: Jump, size: Size) {
        let data = [0, size.in_bytes(), 0]; // no offset, specified size, relative

        stmts.push(Stmt::zeroed(size));
        stmts.push(reloc.encode(&data));
    }

    fn default_align(&self) -> u8 {
        0x90
    }
}

impl AssembleX86 for State<'_> {
    fn compile_instruction(&mut self, arch: &Archx86, instruction: InstructionX86) -> Result<(), Error> {
        let InstructionX86 { inst, args } = instruction;

        let ctx = Context {
            state: self,
            mode: X86Mode::Protected,
            features: arch.features,
        };

        compiler::compile_instruction(ctx, inst, args)
    }

    fn build_instruction(&mut self, _: &Archx86, _: InstructionX86) -> Result<(), Error>
        where Self: BasicExprBuilder
    {
        unreachable!("Statically uncallable, Self is not BasicExprBuilder")
    }
}
