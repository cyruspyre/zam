use crate::{block::Block, expression::term::Term, source::Source};

use super::Statement;

fn _break() -> Block {
    Block {
        fun: Vec::new(),
        ext: Vec::new(),
        stk: Vec::new(),
        stm: vec![Statement::Break(String::new())],
    }
}

impl Source {
    pub fn r#loop(&mut self, parent_stm: &mut Vec<Statement>, typ: &str) -> Statement {
        let mut stm = Vec::new();

        if let Some(v) = match typ {
            "for" => {
                let (val, exp) = (
                    format!("{}{}", stm.len(), self.identifier(false)),
                    stm.len().to_string(),
                );
                let nullable = format!("_{val}");
                self.expect(&["in"]);

                parent_stm.push(Statement::Variable {
                    name: exp.clone(),
                    typ: None,
                    val: self.exp('{', true).0,
                    cte: false,
                });

                for ele in [val.clone(), nullable.clone()] {
                    parent_stm.push(Statement::Variable {
                        name: ele.clone(),
                        typ: None,
                        val: Vec::new(),
                        cte: false,
                    });
                }

                stm.push(Statement::Expression(vec![
                    Term::Identifier(nullable.clone()),
                    Term::Assign,
                    Term::Identifier(exp),
                    Term::Access("next".into()),
                    Term::Call(Vec::new()),
                ]));

                Some(Statement::Conditional {
                    cond: vec![(
                        vec![Term::Identifier(nullable.clone()), Term::Eq, Term::Null],
                        _break(),
                    )],
                    default: Some(Block {
                        fun: Vec::new(),
                        ext: Vec::new(),
                        stk: Vec::new(),
                        stm: vec![Statement::Expression(vec![
                            Term::Identifier(val),
                            Term::Assign,
                            Term::Deref,
                            Term::Identifier(nullable),
                        ])],
                    }),
                })
            }
            "while" => Some(Statement::Conditional {
                cond: vec![(vec![Term::Neg, Term::Group(self.exp('{', true).0)], _break())],
                default: None,
            }),
            _ => None,
        } {
            stm.push(v)
        }

        Statement::Loop(self._block(false, stm))
    }
}