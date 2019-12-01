mod ast;
mod compiler;
pub mod parser;
mod debug;
mod x64data;

use crate::State;
use crate::arch::{Arch, Error, BasicExprBuilder};
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

pub struct InstructionX64 {
    pub inst: ast::Instruction,
    pub args: Vec<ast::CleanArg>,
}

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

        stmts.push(Stmt::Const(0, size));
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

        stmts.push(Stmt::Const(0, size));
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
