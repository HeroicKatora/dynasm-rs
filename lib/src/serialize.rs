use byteorder::{ByteOrder, LittleEndian};

use crate::common::{Size, Stmt, delimited};


/// Converts a sequence of abstract Statements to actual tokens
pub fn serialize(name: &TokenTree, stmts: Vec<Stmt>) -> TokenStream {
    // first, try to fold constants into a byte stream
    let mut folded_stmts = Vec::new();
    let mut const_buffer = Vec::new();
    for stmt in stmts {
        match stmt {
            Stmt::Const(value, size) => {
                match size {
                    Size::BYTE => const_buffer.push(value as u8),
                    Size::WORD => {
                        let mut buffer = [0u8; 2];
                        LittleEndian::write_u16(&mut buffer, value as u16);
                        const_buffer.extend(&buffer);
                    },
                    Size::DWORD => {
                        let mut buffer = [0u8; 4];
                        LittleEndian::write_u32(&mut buffer, value as u32);
                        const_buffer.extend(&buffer);
                    },
                    Size::QWORD => {
                        let mut buffer = [0u8; 8];
                        LittleEndian::write_u64(&mut buffer, value as u64);
                        const_buffer.extend(&buffer);
                    },
                    _ => unimplemented!()
                }
            },
            Stmt::Extend(data) => {
                const_buffer.extend(data);
            },
            s => {
                // empty the const buffer
                if !const_buffer.is_empty() {
                    folded_stmts.push(Stmt::Extend(const_buffer));
                    const_buffer = Vec::new();
                }
                folded_stmts.push(s);
            }
        }
        while const_buffer.len() > 32 {
            let new_buffer = const_buffer.split_off(32);
            folded_stmts.push(Stmt::Extend(const_buffer));
            const_buffer = new_buffer;
        }
    }
    if !const_buffer.is_empty() {
        folded_stmts.push(Stmt::Extend(const_buffer));
    }

    // and now do the final output pass in one go
    let mut output = TokenStream::new();

    for stmt in folded_stmts {
        let (method, args) = match stmt {
            Stmt::Const(_, _) => unreachable!(),
            Stmt::ExprUnsigned(expr, Size::BYTE)  => ("push",     vec![expr]),
            Stmt::ExprUnsigned(expr, Size::WORD)  => ("push_u16", vec![expr]),
            Stmt::ExprUnsigned(expr, Size::DWORD) => ("push_u32", vec![expr]),
            Stmt::ExprUnsigned(expr, Size::QWORD) => ("push_u64", vec![expr]),
            Stmt::ExprUnsigned(_, _) => unimplemented!(),
            Stmt::ExprSigned(  expr, Size::BYTE)  => ("push_i8",  vec![expr]),
            Stmt::ExprSigned(  expr, Size::WORD)  => ("push_i16", vec![expr]),
            Stmt::ExprSigned(  expr, Size::DWORD) => ("push_i32", vec![expr]),
            Stmt::ExprSigned(  expr, Size::QWORD) => ("push_i64", vec![expr]),
            Stmt::ExprSigned(_, _) => unimplemented!(),
            Stmt::Extend(data)     => ("extend", vec![proc_macro2::Literal::byte_string(&data).into()]),
            Stmt::ExprExtend(expr) => ("extend", vec![expr]),
            Stmt::Align(expr, with)      => ("align", vec![expr, with]),
            Stmt::GlobalLabel(n) => ("global_label", vec![expr_string_from_ident(&n)]),
            Stmt::LocalLabel(n)  => ("local_label", vec![expr_string_from_ident(&n)]),
            Stmt::DynamicLabel(expr) => ("dynamic_label", vec![expr]),
            Stmt::GlobalJumpTarget(n,     offset, reloc) => ("global_reloc"  , vec![expr_string_from_ident(&n), offset, reloc]),
            Stmt::ForwardJumpTarget(n,    offset, reloc) => ("forward_reloc" , vec![expr_string_from_ident(&n), offset, reloc]),
            Stmt::BackwardJumpTarget(n,   offset, reloc) => ("backward_reloc", vec![expr_string_from_ident(&n), offset, reloc]),
            Stmt::DynamicJumpTarget(expr, offset, reloc) => ("dynamic_reloc" , vec![expr, offset, reloc]),
            Stmt::BareJumpTarget(expr, reloc)    => ("bare_reloc"    , vec![expr, reloc]),
            Stmt::Stmt(s) => {
                output.extend(quote! {
                    #s ;
                });
                continue;
            }
        };

        // and construct the appropriate method call
        let method = syn::Ident::new(method, Span::call_site());
        output.extend(quote! {
            #name . #method ( #( #args ),* ) ;
        })
    }

    // if we have nothing to emit, expand to nothing. Else, wrap it into a block.
    if output.is_empty() {
        output
    } else {
        quote!{
            {
                #output
            }
        }
    }
}
//
// makes the following
// match size {
//    8 => 3,
//    4 => 2,
//    2 => 1,
//    1 => 0,
//    _ => panic!r("Type size not representable as scale")
//}
pub fn expr_encode_x64_sib_scale(size: &TokenTree) -> TokenTree {
    let span = size.span();

    delimited(quote_spanned! { span=>
        match #size {
            8 => 3,
            4 => 2,
            2 => 1,
            1 => 0,
            _ => panic!("Type size not representable as scale")
        }
    })
}
