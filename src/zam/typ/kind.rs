use std::fmt::Display;

use super::{misc::join, Type};

#[derive(Debug, Clone, Default)]
pub enum TypeKind {
    #[default]
    Unknown,
    Integer {
        bit: u32,
        sign: bool,
    },
    Float(u32),
    ID(String),
    Fn {
        arg: Vec<Type>,
        ret: Box<Type>,
    },
    Tuple(Vec<Type>),
}

impl Display for TypeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = match self {
            TypeKind::Integer { bit, sign } => format!("{}{bit}", if *sign { 'i' } else { 'u' }),
            TypeKind::Float(v) => format!("f{v}"),
            TypeKind::Fn { arg, ret } => format!("fn({}) -> {ret}", join(arg)),
            TypeKind::Tuple(items) => join(items),
            TypeKind::ID(v) => v.into(),
            TypeKind::Unknown => "UNKNOWN".into(),
        };

        f.write_str(&data)
    }
}

impl TypeKind {
    pub fn try_as_number(&mut self, max_bit: u32) -> Option<String> {
        let TypeKind::ID(id) = self else { return None };
        let mut iter = id.chars();
        let Some(pfx) = iter.next() else {
            return None;
        };
        let mut place = 10u32.pow(id.len() as u32 - 2);
        let mut tmp = "size".chars();
        let mut bit = 0;

        while let Some(c) = iter.next() {
            bit += match c {
                _ if c.is_ascii_alphabetic() => match tmp.next() {
                    Some(v) if c == v => continue,
                    _ => return None,
                },
                '0' if bit == 0 => return None,
                _ => match c.to_digit(10) {
                    Some(n) => n * place,
                    _ => return None,
                },
            };
            place /= 10;
        }

        if tmp.next().is_none() {
            if pfx == 'f' {
                return Some("did you mean `f32` or `f64`?".into());
            }

            bit = max_bit
        }

        *self = match pfx {
            'f' if matches!(bit, 32 | 64) => TypeKind::Float(bit),
            'i' | 'u' => match bit {
                1..=64 => TypeKind::Integer {
                    bit,
                    sign: pfx == 's',
                },
                _ => return Some(format!("did you mean `{pfx}{{1..=64}}`?")),
            },
            _ => return None,
        };

        None
    }
}
