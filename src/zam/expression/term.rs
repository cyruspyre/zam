use std::fmt::Display;

use super::{
    super::{fields::Field, Block},
    Expression, PrettyExp, Parser,
};

#[derive(Debug, Clone)]
pub enum Term {
    Void,
    Integer {
        val: u64,
        bit: u32,
        neg: bool,
        rng: [usize; 2],
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
    Struct(Vec<Field<Expression>>),
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
    Div,
    Mod,
    Shl,
    Shr,
    As(String),
    Eq,
    Nq,
    Lt,
    Gt,
    Le,
    Ge,
}

impl Term {
    pub fn check_rng(&self, src: &mut Parser) {
        if let Term::Integer {
            val,
            bit,
            sign,
            neg,
            rng,
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
                src.rng = *rng;
                src.err(&format!(
                    "`{}{bit}` has a range of `{}..={max}`",
                    if *sign { 'i' } else { 'u' },
                    min as i64
                ));
            }
        }
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            Term::Integer { val, bit, sign, .. } => match *sign {
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
            Term::As(v) => format!("as {v}"),
            Term::Identifier(v) => v.into(),
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
