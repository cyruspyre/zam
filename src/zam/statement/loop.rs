use indexmap::IndexMap;

use crate::zam::expression::Expression;

use super::{
    super::{expression::term::Term, Parser},
    Block, Statement,
};

impl Parser {
    pub fn r#loop(&mut self, parent_stm: &mut Vec<Statement>, typ: &str) -> Option<Statement> {
        macro_rules! arr {
            ($($x:expr),+ $(,)?) => {[$($x),+].map(|v| self.span(v)).to_vec()};
        }

        let mut stm = Vec::new();

        if let Some(v) = match typ {
            "for" => {
                let tmp = self.identifier(true)?;
                let [val, exp] = [format!("{}{tmp}", stm.len()), stm.len().to_string()];
                let nullable = format!("_{val}");
                self.expect(&["in"])?;

                parent_stm.push(Statement::Variable {
                    name: self.span(exp.clone()),
                    exp: self.exp('{', true)?.0,
                    cte: false,
                });

                for ele in [val.clone(), nullable.clone()] {
                    parent_stm.push(Statement::Variable {
                        name: self.span(ele),
                        exp: Default::default(),
                        cte: false,
                    });
                }

                stm.push(Statement::Expression(Expression::from(arr![
                    Term::Identifier(nullable.clone()),
                    Term::Assign,
                    Term::Identifier(exp),
                    Term::Access(false),
                    "next".into(),
                    Term::Tuple(Vec::new()),
                ])));

                Some(Statement::Conditional {
                    cond: vec![(
                        Expression::from(arr![
                            Term::Identifier(nullable.clone()),
                            Term::Eq,
                            Term::Null
                        ]),
                        self._break(),
                    )],
                    default: Some(Block {
                        dec: IndexMap::new(),
                        stm: vec![Statement::Expression(Expression::from(arr![
                            Term::Identifier(val),
                            Term::Assign,
                            Term::Deref,
                            Term::Identifier(nullable),
                        ]))],
                    }),
                })
            }
            "while" => Some(Statement::Conditional {
                cond: vec![(
                    Expression::from(arr![Term::Neg, Term::Group(self.exp('{', true)?.0)]),
                    self._break(),
                )],
                default: None,
            }),
            _ => None,
        } {
            stm.push(v)
        }

        Some(Statement::Loop(self._block(false, stm)?))
    }

    fn _break(&self) -> Block {
        Block {
            dec: IndexMap::new(),
            stm: vec![Statement::Break(self.span(String::new()))],
        }
    }
}
