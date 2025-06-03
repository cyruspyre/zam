use std::fmt::{Debug, Display};

use crate::{
    parser::span::Span,
    zam::{
        identifier::Identifier,
        statement::Statement,
        typ::{kind::TypeKind, Type},
        Entity,
    },
};

use super::{
    super::{fields::Fields, Block},
    Expression, Parser,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AssignKind {
    Normal,
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    None,
    Bool(bool),
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
    Array {
        val: Vec<Expression>,
        len: Option<Expression>,
    },
    Block(Block),
    Group(Expression),
    Tuple(Vec<Expression>),
    Struct(Fields<Expression>),
    Generic(Vec<Span<Type>>),
    Identifier(Identifier),
    Access,
    Rng,
    Assign(AssignKind),
    As(Type),
    Null,
    Ref,
    Deref,
    Neg,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Shl,
    Shr,
    BitOr,
    BitAnd,
    Eq,
    Nq,
    Lt,
    Gt,
    Le,
    Ge,
    Or,
    And,
}

impl Term {
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
                src.log.err(format!(
                    "`{}{bit}` has a range of `{}..={max}`",
                    if *sign { 'i' } else { 'u' },
                    min as i64
                ));
            }
        }
    }
}

impl<const N: usize> Into<Term> for [&str; N] {
    fn into(self) -> Term {
        Term::Identifier(Identifier::from(self))
    }
}

impl Into<Term> for &str {
    fn into(self) -> Term {
        Term::Identifier(Identifier::from([self]))
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
            Term::Struct(fields) => format!("{fields:#?}"),
            Term::Tuple(v) => format!(
                "({})",
                v.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Term::As(v) => format!("as {v}"),
            Term::Identifier(v) => v.to_string(),
            Term::String { data, byte } => match byte {
                true => format!("{:?}", data.as_bytes()),
                _ => format!("{data:?}"),
            },
            Term::Block(Block {
                dec, stm, public, ..
            }) => {
                let mut buf = Vec::with_capacity(dec.len() + stm.len());

                for (i, (k, v)) in dec.iter().enumerate() {
                    let tmp = match v {
                        Entity::Variable { exp, cte, .. } => {
                            format!(
                                "{}{} {k}{}{};",
                                match public.binary_search(&i).is_ok() {
                                    true => "pub ",
                                    _ => "",
                                },
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
                        Statement::Variable {
                            id,
                            data: Entity::Variable { exp, cte, .. },
                        } => {
                            format!(
                                "{} {id}{}{};",
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
                Term::Access => ".",
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
