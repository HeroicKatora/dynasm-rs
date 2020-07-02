A pure rust assembler, not a JIT. Used within direct-asm for maximum control
over assembly.

This crate implements uniform parsing of assembly into directives, labels,
expression and backends for some architectures (x86, x86_64, aarch64 WIP). It's
not exactly Intel syntax (but somewhat close) since it should be possible to
embed expressions from the environment, which we must treat as arbitrary opaque
values that we can not manipulate directly.

There is no global state and we don't assume to be executed within a proc-macro
but that is one possible embedding. In that case we can _combine_ expressions
but not evaluate them. So, e.g. to embed some `A: u8` whose three lowest bits
give an index into the top three bits of an output byte, we ask the environment
to generate a new expression for `(A & 0x7) << 5`. This yields the right result
after const eval without having inspected the value of `A` ourselves. With
enough of these combinators we can do all necessary operations for assembling. 

This is a heavy work in progress, any contribution is welcome. Parser, new
arch, better diagnostics.

Restriction: Inserting pointers and other relocations _from_ the environment is
not easy or outright impossible. We can directly insert a pointer as is which
would not permit a pure byte slice as an output and requires an actual `struct`
or complicated enum instead. However, the pointer's location must later be
replaced by the linker and it has a replacement value describing the location
in MIR const eval so we can inspect its bytes or insert compound arithmetic
expressions of it. That must be evaluated at runtime. (In a completely stupid
move we would modify the linker script to do that arithmetic but we can't go
that crazy yet, the basics should work before that. Another option would be a
compiler interface and there is some sympathy for it but nothing official or
concrete.)
