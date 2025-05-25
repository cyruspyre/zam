use std::{fmt::Display, u32};

use crate::{
    misc::{Ref, RefMut},
    zam::{identifier::Identifier, Entity},
};

use super::{misc::join, Type};

#[derive(Debug, Clone, Default, PartialEq)]
pub enum TypeKind {
    #[default]
    Unknown,
    None,
    Bool,
    Integer {
        bit: u32,
        sign: bool,
    },
    Float(u32),
    ID(Identifier),
    Entity {
        id: Ref<Identifier>,
        data: RefMut<Entity>,
    },
    Fn {
        arg: Vec<Type>,
        ret: Box<Type>,
    },
    Tuple(Vec<Type>),
}

impl Display for TypeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = match self {
            TypeKind::None => "()".into(),
            TypeKind::Bool => "bool".into(),
            TypeKind::Integer { bit, sign } => match bit {
                0 => format!("<{}integer>", if *sign { "signed_" } else { "" }),
                _ => format!(
                    "{}{}",
                    if *sign { 'i' } else { 'u' },
                    match *bit {
                        u32::MAX => "size".into(),
                        _ => bit.to_string(),
                    },
                ),
            },
            TypeKind::Float(v) => match v {
                0 => "<float>".into(),
                _ => format!("f{v}"),
            },
            TypeKind::Fn { arg, ret } => format!("fn({}) -> {ret}", join(arg)),
            TypeKind::Tuple(items) => join(items),
            TypeKind::ID(v) => v.to_string(),
            TypeKind::Unknown => "UNKNOWN".into(),
            TypeKind::Entity { id, .. } => id.to_string(),
        };

        f.write_str(&data)
    }
}

impl TypeKind {
    pub fn try_as_number(&mut self) -> Option<String> {
        let id = match self {
            TypeKind::ID(id) if id.is_qualified() => id.leaf_name(),
            _ => return None,
        };
        let mut iter = id.chars();
        let Some(pfx) = iter.next() else {
            return None;
        };
        let mut place = 10u32.pow(id.len().checked_sub(2).unwrap_or_default() as u32);
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

            bit = u32::MAX
        }

        *self = match pfx {
            'f' if matches!(bit, 32 | 64) => TypeKind::Float(bit),
            'i' | 'u' => match bit {
                1..=64 | u32::MAX => TypeKind::Integer {
                    bit,
                    sign: pfx == 'i',
                },
                _ => return Some(format!("did you mean `{pfx}{{1..=64}}`?")),
            },
            _ => return None,
        };

        None
    }
}
