//! This module contains various infrastructure that is common across all assembler backends

/// Enum representing the result size of a value/expression/register/etc in bytes.
/// Uses the NASM syntax for sizes (a word is 16 bits)
#[derive(Debug, PartialOrd, PartialEq, Ord, Eq, Hash, Clone, Copy)]
pub enum Size {
    BYTE  = 1,
    WORD  = 2,
    DWORD = 4,
    FWORD = 6,
    QWORD = 8,
    PWORD = 10,
    OWORD = 16,
    HWORD = 32,
}

impl Size {
    pub fn in_bytes(self) -> u8 {
        self as u8
    }

    pub fn as_literal(self) -> &'static str {
        match self {
            Size::BYTE  => "i8",
            Size::WORD  => "i16",
            Size::DWORD => "i32",
            Size::FWORD => "i48",
            Size::QWORD => "i64",
            Size::PWORD => "i80",
            Size::OWORD => "i128",
            Size::HWORD => "i256",
        }
    }
}

/**
 * Jump types
 */
#[derive(Debug, Clone)]
pub struct Jump {
    pub kind: JumpKind,
    pub offset: Option<Expr>,
}

#[derive(Debug, Clone)]
pub enum JumpKind {
    // note: these symbol choices try to avoid stuff that is a valid starting symbol for parse_expr
    // in order to allow the full range of expressions to be used. the only currently existing ambiguity is
    // with the symbol <, as this symbol is also the starting symbol for the universal calling syntax <Type as Trait>.method(args)
    Global(Ident),   // -> label (["+" "-"] offset)?
    Backward(Ident), //  > label (["+" "-"] offset)?
    Forward(Ident),  //  < label (["+" "-"] offset)?
    Dynamic(Expr),   // =>expr | => (expr) (["+" "-"] offset)?
    Bare(Expr)       // jump to this address
}

impl Jump {
    pub fn new(kind: JumpKind, offset: Option<Expr>) -> Jump {
        Jump {
            kind,
            offset,
        }
    }

    pub fn encode(self, data: &[u8]) -> Stmt {
        let span = self.span();

        let offset = self.offset.into(); 

        let data = serialize::expr_tuple_of_u8s(span, data);
        match self.kind {
            JumpKind::Global(ident) => Stmt::GlobalJumpTarget(ident, offset, data),
            JumpKind::Backward(ident) => Stmt::BackwardJumpTarget(ident, offset, data),
            JumpKind::Forward(ident) => Stmt::ForwardJumpTarget(ident, offset, data),
            JumpKind::Dynamic(expr) => Stmt::DynamicJumpTarget(expr.into(), offset, data),
            JumpKind::Bare(expr) => Stmt::BareJumpTarget(expr.into(), data),
        }
    }

    pub fn span(&self) -> Span {
        match &self.kind {
            JumpKind::Global(ident) => ident.span(),
            JumpKind::Backward(ident) => ident.span(),
            JumpKind::Forward(ident) => ident.span(),
            JumpKind::Dynamic(expr) => expr.span(),
            JumpKind::Bare(expr) => expr.span(),
        }
    }
}


/// An abstract representation of a dynasm runtime statement to be emitted
#[derive(Debug, Clone)]
pub enum Stmt {
    // simply push data into the instruction stream. unsigned
    Const(u64, Size),
    // push data that is stored inside of an expression. unsigned
    ExprUnsigned(TokenTree, Size),
    // push signed data into the instruction stream. signed
    ExprSigned(TokenTree, Size),

    // extend the instruction stream with unsigned bytes
    Extend(Vec<u8>),
    // extend the instruction stream with unsigned bytes
    ExprExtend(TokenTree),
    // align the instruction stream to some alignment
    Align(TokenTree, TokenTree),

    // label declarations
    GlobalLabel(Ident),
    LocalLabel(Ident),
    DynamicLabel(TokenTree),

    // and their respective relocations (as expressions as they differ per assembler)
    GlobalJumpTarget(Ident, JumpOffset, Vec<u8>),
    ForwardJumpTarget(Ident, JumpOffset, Vec<u8>),
    BackwardJumpTarget(Ident, JumpOffset, Vec<u8>),
    DynamicJumpTarget(JumpOffset, JumpOffset, Vec<u8>),
    BareJumpTarget(JumpOffset, Vec<u8>),

    // a random statement that has to be inserted between assembly hunks
    Stmt(TokenTree),
}

pub enum JumpOffset {
    Zero,
    Injected(Expr),
}

#[derive(Debug, Clone)]
pub struct Ident {
    pub name: String,
}

/// An expression that will be inserted by the caller.
#[derive(Debug, Clone, Copy)]
pub struct Expr {
    pub idx: usize,
}

// convenience methods
impl Stmt {
    #![allow(dead_code)]

    pub fn u8(value: u8) -> Stmt {
        Stmt::Const(u64::from(value), Size::BYTE)
    }

    pub fn u16(value: u16) -> Stmt {
        Stmt::Const(u64::from(value), Size::WORD)
    }

    pub fn u32(value: u32) -> Stmt {
        Stmt::Const(u64::from(value), Size::DWORD)
    }

    pub fn u64(value: u64) -> Stmt {
        Stmt::Const(value, Size::QWORD)
    }
}

/// Create a bitmask with `scale` bits set
pub fn bitmask(scale: u8) -> u32 {
    1u32.checked_shl(u32::from(scale)).unwrap_or(0).wrapping_sub(1)
}


/// Create a bitmask with `scale` bits set
pub fn bitmask64(scale: u8) -> u64 {
    1u64.checked_shl(u32::from(scale)).unwrap_or(0).wrapping_sub(1)
}

impl From<Option<Expr>> for JumpOffset {
    fn from(val: Option<Expr>) -> JumpOffset {
        match val {
            None => JumpOffset::Zero,
            Some(expr) => JumpOffset::Injected(expr),
        }
    }
}

impl From<Expr> for JumpOffset {
    fn from(val: Option<Expr>) -> JumpOffset {
        expr => JumpOffset::Injected(expr),
    }
}
