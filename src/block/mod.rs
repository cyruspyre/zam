use crate::{
    external::External, function::Function, r#struct::Struct, source::Source, statement::Statement,
};

#[derive(Debug)]
pub struct Block {
    pub fun: Vec<Function>,
    pub ext: Vec<External>,
    pub stk: Vec<Struct>,
    pub stm: Vec<Statement>,
}

impl Source {
    pub fn block(&mut self, global: bool) -> Block {
        if !global {
            self.expect(&['{']);
            self.ensure_closed('}');
        }

        let mut fun = Vec::new();
        let mut ext = Vec::new();
        let mut stk = Vec::new();
        let mut stm = Vec::new();
        let de = match self.de.last() {
            Some(n) => n - 1,
            _ => 0,
        };

        'one: loop {
            if self.idx == de {
                self.de.pop();
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
                match tmp {
                    "fn" => fun.push(self.fun()),
                    "struct" => stk.push(self.strukt()),
                    _ => break 'two,
                }

                continue 'one;
            }

            if global {
                self.err("expected keyword of global context")
            } else if stamp == de {
                continue;
            }

            stm.push(match tmp {
                "let" | "cte" => self.var(tmp == "cte"),
                "if" => self.cond(),
                _ => {
                    self.idx = stamp;

                    let (exp, used) = self.exp(';');

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

        Block { fun, ext, stk, stm }
    }
}