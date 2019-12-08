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

/// A number representation (sign and size).
#[derive(Debug, PartialOrd, PartialEq, Ord, Eq, Hash, Clone, Copy)]
pub struct NumericRepr {
    pub size: Size,
    pub signed: bool,
}

/// An integral value in a particular `Numeric` representation.
#[derive(Debug, PartialOrd, PartialEq, Ord, Eq, Hash, Clone, Copy)]
pub struct Number {
    /// The bit representation of the number.
    /// TODO: comment on the actual representation chosen.
    value: u64,
    repr: NumericRepr,
}

impl Size {
    pub const fn in_bytes(self) -> u8 {
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

impl NumericRepr {
    pub const U8: NumericRepr = NumericRepr::unsigned(Size::BYTE);
    pub const I8: NumericRepr = NumericRepr::signed(Size::BYTE);
    pub const U16: NumericRepr = NumericRepr::unsigned(Size::WORD);
    pub const I16: NumericRepr = NumericRepr::signed(Size::WORD);
    pub const U32: NumericRepr = NumericRepr::unsigned(Size::DWORD);
    pub const I32: NumericRepr = NumericRepr::signed(Size::DWORD);
    pub const U64: NumericRepr = NumericRepr::unsigned(Size::QWORD);
    pub const I64: NumericRepr = NumericRepr::signed(Size::QWORD);

    pub const fn signed(size: Size) -> Self {
        NumericRepr { size, signed: true }
    }

    pub const fn unsigned(size: Size) -> Self {
        NumericRepr { size, signed: false }
    }
}

impl Number {
    /// Cast a short constant to a specific representation.
    pub const fn from_u64_and_size(val: u64, size: Size) -> Self {
        Self::from_u64_and_repr(val, NumericRepr::unsigned(size))
    }

    pub const fn from_u64_and_repr(value: u64, repr: NumericRepr) -> Self {
        Number { value, repr }
    }

    pub const fn repr(self) -> NumericRepr {
        self.repr
    }

    pub fn byte(val: u8) -> Self {
        Self::from_u64_and_size(val.into(), Size::BYTE)
    }

    pub fn word(val: u16) -> Self {
        Self::from_u64_and_size(val.into(), Size::WORD)
    }

    pub fn dword(val: u32) -> Self {
        Self::from_u64_and_size(val.into(), Size::DWORD)
    }

    pub fn qword(val: u64) -> Self {
        Self::from_u64_and_size(val.into(), Size::QWORD)
    }

    pub fn as_u8(self) -> u8 {
        self.cast_as(NumericRepr::unsigned(Size::BYTE)).value as u8
    }

    pub fn as_i8(self) -> i8 {
        self.cast_as(NumericRepr::signed(Size::BYTE)).value as i8
    }

    pub fn as_u16(self) -> u16 {
        self.cast_as(NumericRepr::unsigned(Size::WORD)).value as u16
    }

    pub fn as_i16(self) -> i16 {
        self.cast_as(NumericRepr::signed(Size::WORD)).value as i16
    }

    pub fn as_u32(self) -> u32 {
        self.cast_as(NumericRepr::unsigned(Size::DWORD)).value as u32
    }

    pub fn as_i32(self) -> i32 {
        self.cast_as(NumericRepr::signed(Size::DWORD)).value as i32
    }

    pub fn as_u64(self) -> u64 {
        self.cast_as(NumericRepr::unsigned(Size::QWORD)).value as u64
    }

    pub fn as_i64(self) -> i64 {
        self.cast_as(NumericRepr::signed(Size::DWORD)).value as i64
    }

    /// Perform a cast in 2-complement.
    ///
    /// Casts work like Rust `as` coercion. A sign extension is performed when the source is
    /// signed, else the number is zero extended.
    // FIXME: test coverage!
    pub fn cast_as(mut self, repr: NumericRepr) -> Number {
        // Just use the value, it is stored with sign/zero extension.
        self.repr = repr;
        // Adjust sign extension if necessary now.
        self.correct_extension_bits_for_sign();
        self
    }

    /// Do a value preserving (`TryFrom`) conversion.
    ///
    /// This is not the same as lossless, `u32` and `i32` can be converted without loss but do not
    /// preserve the values.
    pub fn convert(self, repr: NumericRepr) -> Option<Number> {
        let cast = self.cast_as(repr);

        let max = self.repr_of_max().min(cast.repr_of_max());
        let below_min = self.repr_below_min().max(cast.repr_below_min());

        if cast.value <= max && cast.value > below_min {
            Some(cast)
        } else {
            None
        }
    }

    pub fn make_signed(mut self, signed: bool) -> Number {
        self.repr.signed = signed;
        self.correct_extension_bits_for_sign();
        self
    }

    /// Resize, keeping the same signedness.
    pub const fn resize(self, size: Size) -> Number {
        // Because resizing does not change signedness this yields correct extension bits.
        Number {
            value: self.value,
            repr: NumericRepr { size, signed: self.repr.signed },
        }
    }

    /// The value bitmask for the size.
    fn mask(self) -> u64 {
        use core::convert::TryInto;
        #[allow(non_snake_case)]
        let ALL_BITS: u8 = core::mem::size_of::<u64>().try_into().unwrap();

        let len: u8 = self.byte_len() * 8;
        (!0u64) >> ALL_BITS.checked_sub(len).unwrap()
    }

    fn byte_len(self) -> u8 {
        self.repr.size.in_bytes()
    }

    /// The maximum value representation.
    /// Used to check the value range in unsigned representation of any length.
    fn repr_of_max(self) -> u64 {
        self.mask() ^ (if self.repr.signed { self.sign_bit() } else { 0 })
    }

    /// The representation below minimum value.
    /// Used to check the value range in unsigned representation of any length.
    fn repr_below_min(self) -> u64 {
        if self.repr.signed {
            ((!0u64) ^ self.repr_of_max()) - 1
        } else {
            0
        }
    }

    fn sign_bit(self) -> u64 {
        let right_shift = (self.byte_len() * 8) - 1;
        1 << right_shift
    }

    fn is_sign_bit_set(self) -> bool {
        self.value & self.sign_bit() != 0
    }

    /// Fix the sign extension after a cast.
    fn correct_extension_bits_for_sign(&mut self) {
        if self.repr.signed && self.is_sign_bit_set() {
            self.value |= !self.mask();
        } else {
            self.value &= self.mask();
        }
    }
}

/// A value in a list of constants.
#[derive(Debug, Clone)]
pub enum Const {
    /// Add constant through applying some relocation.
    Relocate(Jump),

    /// Add a simple value.
    Value(Expr),
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
    Bare(Value)      // jump to this address
}

impl Jump {
    pub fn new(kind: JumpKind, offset: Option<Expr>) -> Jump {
        Jump {
            kind,
            offset,
        }
    }

    pub fn encode(self, data: &[u8]) -> Stmt {
        let offset = self.offset.into(); 
        let data = data.to_vec();
        match self.kind {
            JumpKind::Global(ident) => Stmt::GlobalJumpTarget(ident, offset, data),
            JumpKind::Backward(ident) => Stmt::BackwardJumpTarget(ident, offset, data),
            JumpKind::Forward(ident) => Stmt::ForwardJumpTarget(ident, offset, data),
            JumpKind::Dynamic(expr) => Stmt::DynamicJumpTarget(expr.into(), offset, data),
            JumpKind::Bare(expr) => Stmt::BareJumpTarget(expr.into(), data),
        }
    }
}


/// An abstract representation of a dynasm runtime statement to be emitted
#[derive(Debug, Clone)]
pub enum Stmt {
    // push integral data with arbitrary size.
    Const(Value),

    // extend the instruction stream with unsigned bytes
    Extend(Vec<u8>),
    // extend the instruction stream with unsigned bytes
    ExprExtend(Value),
    // align the instruction stream to some alignment
    // the second is the actual alignment and might be a platform default, hence computed by the
    // assembler library itself instead of a user defined expression.
    Align(Expr, Value),

    // label declarations
    GlobalLabel(Ident),
    LocalLabel(Ident),
    DynamicLabel(Expr),

    // and their respective relocations (as expressions as they differ per assembler)
    GlobalJumpTarget(Ident, JumpOffset, Vec<u8>),
    ForwardJumpTarget(Ident, JumpOffset, Vec<u8>),
    BackwardJumpTarget(Ident, JumpOffset, Vec<u8>),
    DynamicJumpTarget(JumpOffset, JumpOffset, Vec<u8>),
    BareJumpTarget(JumpOffset, Vec<u8>),

    // a random statement that has to be inserted between assembly hunks
    Stmt(Expr),
}

/// A value that is specifically for jump offset use.
/// Slightly more specialized than `Value` since the only non-computed value is if elided.
#[derive(Debug, Clone, Copy)]
pub enum JumpOffset {
    Zero,
    Injected(Value),
}

/// An identifier.
#[derive(Debug, Clone)]
pub struct Ident {
    pub name: String,
}

/// An expression that will be inserted by the caller.
#[derive(Debug, Clone, Copy)]
pub struct Expr {
    /// An index generated by the library user, uniquely identifying this expression.
    pub idx: usize,
    /// Indicate the representation for this numeric expression. In the input, this is used by the
    /// caller to indicate the current type (or the smallest coercible one) while the output uses
    /// it to inform the caller of the final cast to use.
    pub repr: NumericRepr,
}

/// A dynamically or statically computed value.
///
/// To produce valid binary it is mostly only important to know the correct width of the output but
/// the value itself can be computed by the caller. This allows freedom on parsing without
/// requiring the assembler core (this library) to implement an arbitrary expression evaluator. In
/// particular, the evaluation can even be further delayed by the caller and left to `rustc`.
#[derive(Debug, Clone, Copy)]
pub enum Value {
    /// A constant number.
    Number(Number),
    /// An external expression of the caller.
    Expr(Expr),
}

// convenience methods
impl Stmt {
    pub fn u8(value: u8) -> Stmt {
        Stmt::Const(Value::Byte(value))
    }

    pub fn u16(value: u16) -> Stmt {
        Stmt::Const(Value::Word(value))
    }

    pub fn u32(value: u32) -> Stmt {
        Stmt::Const(Value::Dword(value))
    }

    pub fn u64(value: u64) -> Stmt {
        Stmt::Const(Value::Qword(value))
    }

    /// Zeroed bytes of a numeric size.
    pub fn zeroed(size: Size) -> Self {
        let nr = Number::from_u64_and_size(0, size);
        Stmt::Const(Value::Number(nr))
    }
}

impl Ident {
    pub fn to_string(self) -> String {
        self.name
    }
}

impl Value {
    pub fn Byte(val: u8) -> Self {
        Value::Number(Number::byte(val))
    }

    pub fn Word(val: u16) -> Self {
        Value::Number(Number::word(val))
    }

    pub fn Dword(val: u32) -> Self {
        Value::Number(Number::dword(val))
    }

    pub fn Qword(val: u64) -> Self {
        Value::Number(Number::qword(val))
    }

    pub fn repr(self) -> NumericRepr {
        match self {
            Value::Number(nr) => nr.repr,
            Value::Expr(expr) => expr.repr,
        }
    }

    pub fn convert(self, repr: NumericRepr) -> Option<Self> {
        Some(match self {
            Value::Number(nr) => Value::Number(nr.convert(repr)?),
            Value::Expr(expr) => Value::Expr(Expr { idx: expr.idx, repr }),
        })
    }

    pub fn size(self) -> Size {
        self.repr().size
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
            Some(expr) => JumpOffset::Injected(expr.into()),
        }
    }
}

impl From<Expr> for JumpOffset {
    fn from(expr: Expr) -> JumpOffset {
        JumpOffset::Injected(expr.into())
    }
}

impl From<Value> for JumpOffset {
    fn from(val: Value) -> JumpOffset {
        JumpOffset::Injected(val)
    }
}

impl From<&'_ Expr> for JumpOffset {
    fn from(expr: &'_ Expr) -> JumpOffset {
        JumpOffset::Injected((*expr).into())
    }
}

impl From<u8> for Value {
    fn from(val: u8) -> Value {
        Value::Byte(val)
    }
}

impl From<Expr> for Value {
    fn from(expr: Expr) -> Value {
        Value::Expr(expr)
    }
}

impl From<&'_ Expr> for Value {
    fn from(expr: &'_ Expr) -> Value {
        Value::Expr(*expr)
    }
}

impl From<Value> for Stmt {
    fn from(val: Value) -> Self {
        Stmt::Const(val)
    }
}

impl From<&'_ Value> for Stmt {
    fn from(val: &'_ Value) -> Self {
        Stmt::Const(*val)
    }
}
