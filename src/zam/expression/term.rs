use std::fmt::Display;

use crate::parser::span::Identifier;

use super::{
    super::{fields::Field, Block},
    Expression, Parser,
};

#[derive(Debug, Clone)]
pub enum Term {
    None,
    Integer {
        val: u64,
        /// ## Special Cases
        /// `0` -> Infer bitness from preceder. If it's the first term, it should be `32`
        ///
        /// `u32::MAX` -> Target's native pointer size
        bit: u32,
        neg: bool,
        /// `None` -> Infer signedness from preceder. If it's the first term, it should be signed
        ///
        /// `Some(true)` -> Signed
        ///
        /// `Some(false)` -> Unsigned
        sign: Option<bool>,
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
    Struct(Vec<Field<Expression>>),
    Identifier(Identifier),
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
    Div,
    Mod,
    Shl,
    Shr,
    As(Identifier),
    Eq,
    Nq,
    Lt,
    Gt,
    Le,
    Ge,
}

impl Term {
    #[inline]
    pub fn valid_first_term(&self) -> bool {
        match self {
            Term::Integer { .. }
            | Term::Float { .. }
            | Term::Char(_)
            | Term::String { .. }
            | Term::Rng
            | Term::Block(_)
            | Term::Ref
            | Term::Deref
            | Term::Group(_)
            | Term::Identifier(_)
            | Term::None
            | Term::Null
            | Term::Struct(_)
            | Term::Tuple(_) => true,
            _ => false,
        }
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
            let sign = sign.unwrap();
            let max = 2u64
                .wrapping_pow(bit - if sign { 1 } else { 0 })
                .wrapping_sub(1);
            let min = if sign { u64::MAX - max } else { 0 };
            let err = !if sign {
                *val >= min && *neg || *val <= max && !neg
            } else {
                *val >= min && *val <= max && !neg
            };

            if err {
                src.err(format!(
                    "`{}{bit}` has a range of `{}..={max}`",
                    if sign { 'i' } else { 'u' },
                    min as i64
                ));
            }
        }
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            Term::Integer { val, bit, sign, .. } => match sign.unwrap() {
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
            Term::Group(v) => format!("({})", v.to_string()),
            Term::As(v) => format!("as {}", **v),
            Term::Identifier(v) => v.data.clone(),
            _ => match self {
                Term::Access(v) => match v {
                    true => "::",
                    _ => ".",
                },
                Term::Add => "+",
                Term::Sub => "-",
                Term::Div => "/",
                Term::Eq => "==",
                _ => "IDK",
            }
            .into(),
        };

        f.write_str(&out)
    }
}
