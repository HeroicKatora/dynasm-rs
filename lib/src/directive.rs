use std::collections::hash_map::Entry;

use crate::common::{Const, Expr, Stmt, Size, Value};
use crate::arch;
use crate::DynasmData;

pub enum Directive {
    /// Set the architcture.
    Arch(String),
    /// Activate an architecture feature, or none to remove all.
    Feature(Vec<String>),
    /// Directly add some inline data words.
    Data(Size, Vec<Const>),
    /// Add some byte directly to the assembled data.
    Byte(Expr),
    /// Perform an alignment.
    Align {
        value: Expr,
        with: Option<Expr>,
    },
    Alias {
        /// The alias to use.
        alias: String,
        /// The target register which is given an alias.
        reg: String,
    },
    /// A direct expression to add as bytes to the output.
    Expr(Expr),
}

pub enum MalformedDirectiveError {
    /// The architecture that was set was not recognized.
    UnknownArchitecture(String),

    /// The feature at the index was unknown.
    UnknownFeature {
        /// The index, to match to an input span for example.
        idx: usize,
        /// The bad feature.
        what: String,
    },

    DuplicateAlias {
        /// The name that has already been aliased.
        reused: String,
    },

    /// Not a recognized directive.
    UnknownDirective,
}

pub(crate) fn evaluate_directive(file_data: &mut DynasmData, stmts: &mut Vec<Stmt>, directive: &Directive)
    -> Result<(), MalformedDirectiveError>
{
    match directive {
        // TODO: oword, qword, float, double, long double
        Directive::Arch(arch) => {
            // ; .arch ident
            if let Some(a) = arch::from_str(&arch) {
                file_data.current_arch = a;
            } else {
                return Err(MalformedDirectiveError::UnknownArchitecture(arch.to_string()));
            }
        },
        Directive::Feature(features) => {
            // ;.feature none  cancels all features
            if features.len() == 1 && features[0] == "none" {
                file_data.current_arch.set_features(&[]);
            } else {
                file_data.current_arch.set_features(features);
            }
        },
        // ; .byte (expr ("," expr)*)?
        Directive::Data(size, consts) => {
            directive_const(file_data, stmts, &consts, *size);
        },
        Directive::Byte(expr) => {
            // ; .bytes expr
            stmts.push(Stmt::ExprExtend(expr.into()));
        },
        Directive::Align { value, with } => {
            // ; .align expr ("," expr)
            let with = if let Some(with) = with {
                Value::Expr(*with)
            } else {
                let with = file_data.current_arch.default_align();
                Value::Byte(with)
            };

            stmts.push(Stmt::Align(*value, with));
        },
        Directive::Alias { alias, reg, } => {
            // ; .alias ident, ident
            match file_data.aliases.entry(alias.clone()) {
                Entry::Occupied(_) => {
                    return Err(MalformedDirectiveError::DuplicateAlias {
                        reused: alias.clone(),
                    });
                },
                Entry::Vacant(v) => {
                    v.insert(reg.clone());
                }
            }
        },
        d => {
            // unknown directive. skip ahead until we hit a ; so the parser can recover
            return Err(MalformedDirectiveError::UnknownDirective);
        }
    }

    Ok(())
}

fn directive_const(file_data: &mut DynasmData, stmts: &mut Vec<Stmt>, values: &[Const], size: Size) {
    for value in values {
        match value {
            Const::Relocate(jump) => {
                file_data.current_arch.handle_static_reloc(stmts, jump.clone(), size);
            },
            Const::Value(expr) => {
                stmts.push(Stmt::ExprSigned(expr.into(), size));
            },
        }
    }
}
