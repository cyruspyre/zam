mod function;
mod r#struct;

use std::collections::HashMap;

use crate::parser::{
    log::{Log, Point},
    misc::Span,
};

use super::{
    expression::Expression,
    fields::Field,
    statement::Statement,
    typ::{generic::Generic, Type},
    Parser,
};

#[derive(Debug, Default, Clone)]
pub struct Block {
    pub dec: HashMap<String, Span<Hoistable>>,
    pub stm: Vec<Statement>,
}

/// Similar to `Statement`, but with declarations that are hoisted.
///
/// All the fields are applicable to both global and local scope.
/// Except for `Variable`, which is to be used in global scope only.
#[derive(Debug, Clone)]
pub enum Hoistable {
    Function {
        arg: Vec<Field<Type>>,
        gen: Generic,
        ret: Type,
        block: Option<Block>,
        public: bool,
    },
    Struct {
        gen: Generic,
        fields: Vec<Field<Type>>,
        rng: [usize; 2],
        public: bool,
    },
    Variable {
        typ: Option<Type>,
        val: Expression,
        cte: bool,
        public: bool,
    },
}

impl Parser {
    pub fn block(&mut self, global: bool) -> Option<Block> {
        self._block(global, Vec::new())
    }

    pub fn _block(&mut self, global: bool, mut stm: Vec<Statement>) -> Option<Block> {
        if !global {
            self.expect(&['{'])?;
            self.ensure_closed('}');
        }

        let mut dup = HashMap::new();
        let mut flag = true;
        let mut dec: HashMap<String, Span<Hoistable>> = HashMap::new();
        let stm_ref = unsafe { &mut *(&mut stm as *mut _) };
        let de = match self.de.back() {
            Some(n) => n - 1,
            _ => 0,
        };

        'one: loop {
            if self.idx == de {
                self.de.pop_back();
                self._next();
                break;
            }

            let stamp = self.idx;
            let tmp = self.until_whitespace();
            let tmp = tmp.as_str();

            if tmp.is_empty() {
                break;
            }

            'two: {
                let (k, mut v) = match tmp {
                    "fn" => self.fun()?,
                    "struct" => self.strukt()?,
                    // "use" => ,
                    // "extern" => self.ext(&mut ext),
                    "pub" => {
                        flag = true;
                        continue 'one;
                    }
                    _ => break 'two,
                };

                if flag {
                    flag = false;
                    match &mut v.data {
                        Hoistable::Function { public, .. }
                        | Hoistable::Struct { public, .. }
                        | Hoistable::Variable { public, .. } => *public = true,
                    }
                }

                if let Some(prev) = dec.get(&k) {
                    dup.entry(k)
                        .or_insert(vec![(prev.rng, Point::Error, "first declared here")])
                        .push((v.rng, Point::Error, ""))
                } else {
                    dec.insert(k, v);
                }

                continue 'one;
            }

            if global {
                self.err("expected keyword of global context")?
            } else if stamp == de {
                continue;
            }

            stm.push(match tmp {
                "let" | "cte" => self.var(tmp == "cte")?,
                "if" => self.cond()?,
                "for" | "loop" | "while" => self.r#loop(stm_ref, tmp)?,
                _ => {
                    self.idx = stamp;

                    let (exp, used) = self.exp(';', false)?;

                    if used {
                        self._next();
                    }

                    if exp.is_empty() {
                        continue;
                    }

                    match used {
                        true => Statement::Expression(exp),
                        _ => Statement::Return(exp),
                    }
                }
            });
        }

        for (k, v) in dup {
            self.log(
                &v,
                Log::Error,
                &format!("identifier `{k}` declared multiple times"),
            );
        }

        Some(Block { dec, stm })
    }
}
