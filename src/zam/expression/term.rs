use std::fmt::Display;

use crate::{
    cfg::Config,
    parser::span::Span,
    zam::{
        block::Hoistable,
        statement::Statement,
        typ::{kind::TypeKind, Type},
    },
};

use super::{
    super::{fields::Fields, Block},
    Expression, Parser,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    None,
    Integer {
        val: u64,
        /// ## Special Cases
        /// `0` -> Infer signedness && bitness from preceder. If it's the first term, it should be `i32`
        ///
        /// `u32::MAX` -> Target's native pointer size
        bit: u32,
        neg: bool,
        /// `None` -> Infer signedness from preceder. If it's the first term, it should be signed
        ///
        /// `Some(true)` -> Signed
        ///
        /// `Some(false)` -> Unsigned
        sign: bool,
    },
    Float {
        val: f64,
        bit: u32,
    },
    Char(char),
    String {
        data: String,
        byte: bool,
    },
    Block(Block),
    Group(Expression),
    Tuple(Vec<Expression>),
    Struct(Fields<Expression>),
    Generic(Vec<Span<Type>>),
    Identifier(String),
    Access(bool),
    Rng,
    Assign,
    Null,
    Ref,
    Deref,
    Neg,
    Add,
    AddAssign,
    Sub,
    Mul,
    Div,
    Mod,
    Shl,
    Shr,
    As(Type),
    Eq,
    Nq,
    Lt,
    Gt,
    Le,
    Ge,
}

impl Term {
    pub fn as_type(&self, from: &Type, cfg: &Config) {
        let tmp = match self {
            Term::Integer { bit, sign, .. } => match bit {
                0 => match from.kind.data {
                    TypeKind::Integer { .. } => from.kind.data.clone(),
                    _ => TypeKind::Integer {
                        bit: 32,
                        sign: true,
                    },
                },
                _ => TypeKind::Integer {
                    bit: if *bit == u32::MAX { cfg.bit } else { *bit },
                    sign: *sign,
                },
            },
            _ => todo!(),
        };

        println!("{tmp:?}");
    }

    pub fn check_rng(&self, src: &mut Parser) {
        if let Term::Integer {
            val,
            bit,
            sign,
            neg,
            ..
        } = self
        {
            let max = 2u64
                .wrapping_pow(bit - if *sign { 1 } else { 0 })
                .wrapping_sub(1);
            let min = if *sign { u64::MAX - max } else { 0 };
            let err = !if *sign {
                *val >= min && *neg || *val <= max && !neg
            } else {
                *val >= min && *val <= max && !neg
            };

            if err {
                src.err(format!(
                    "`{}{bit}` has a range of `{}..={max}`",
                    if *sign { 'i' } else { 'u' },
                    min as i64
                ));
            }
        }
    }
}

impl Into<Term> for &str {
    fn into(self) -> Term {
        Term::Identifier(self.into())
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            Term::Integer { val, bit, sign, .. } => match sign {
                true => {
                    let val = *val as i64;
                    let tmp = format!("{val}i{bit}");

                    match val.is_negative() {
                        true => format!("({tmp})"),
                        _ => tmp,
                    }
                }
                _ => format!("{val}u{bit}"),
            },
            Term::Float { val, bit } => format!("{val:?}f{bit}"),
            Term::Group(v) => format!("({})", v.to_string()),
            Term::Tuple(v) => format!(
                "({})",
                v.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Term::As(v) => format!("as {v}"),
            Term::Identifier(v) => v.clone(),
            Term::String { data, byte } => match byte {
                true => format!("{:?}", data.as_bytes()),
                _ => format!("{data:?}"),
            },
            Term::Block(Block { dec, stm }) => {
                let mut buf = Vec::with_capacity(dec.len() + stm.len());

                for (k, v) in dec {
                    let tmp = match v {
                        Hoistable::Variable {
                            exp, cte, public, ..
                        } => {
                            format!(
                                "{}{} {k}{}{};",
                                if *public { "pub " } else { "" },
                                if *cte { "cte" } else { "let" },
                                match &exp.typ.kind.data {
                                    TypeKind::Unknown => String::new(),
                                    v => format!(": {v}"),
                                },
                                format!(
                                    " = {}",
                                    exp.data
                                        .iter()
                                        .map(|v| v.to_string())
                                        .collect::<Vec<_>>()
                                        .join(" ")
                                )
                            )
                        }
                        _ => todo!(),
                    };

                    buf.push(tmp);
                }

                for v in stm {
                    let tmp = match v {
                        Statement::Variable { name, exp, cte } => {
                            format!(
                                "{} {name}{}{};",
                                if *cte { "cte" } else { "let" },
                                match &exp.typ.kind.data {
                                    TypeKind::Unknown => String::new(),
                                    v => format!(": {v}"),
                                },
                                format!(
                                    " = {}",
                                    exp.data
                                        .iter()
                                        .map(|v| v.to_string())
                                        .collect::<Vec<_>>()
                                        .join(" ")
                                )
                            )
                        }
                        _ => "lol\n".to_string(),
                    };

                    buf.push(tmp);
                }

                format!("{{\n    {}\n}}", buf.join("\n"))
            }
            _ => match self {
                Term::Access(v) => match v {
                    true => "::",
                    _ => ".",
                },
                Term::Add => "+",
                Term::Sub => "-",
                Term::Mul | Term::Deref => "*",
                Term::Div => "/",
                Term::Mod => "%",
                Term::Eq => "==",
                _ => "UNKNOWN",
            }
            .into(),
        };

        f.write_str(&out)
    }
}
