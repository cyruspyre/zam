use crate::zam::{
    block::BlockType,
    expression::{term::AssignKind, Expression},
    identifier::Identifier,
    Entity,
};

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
                let tmp = self.identifier(true, false)?;
                let [val, exp]: [Identifier; 2] =
                    [format!("{}{tmp}", stm.len()), stm.len().to_string()].map(|v| v.into());
                let nullable: Identifier = format!("_{val}").into();
                self.expect(&["in"])?;

                parent_stm.push(Statement::Variable {
                    id: exp.clone(),
                    data: Entity::Variable {
                        exp: self.exp(['{'], true)?.0,
                        cte: false,
                        done: false,
                    },
                });

                for ele in [val.clone(), nullable.clone()] {
                    parent_stm.push(Statement::Variable {
                        id: ele,
                        data: Entity::Variable {
                            exp: Default::default(),
                            cte: false,
                            done: false,
                        },
                    });
                }

                stm.push(Statement::Expression(Expression::from(arr![
                    Term::Identifier(nullable.clone()),
                    Term::Assign(AssignKind::Normal),
                    Term::Identifier(exp),
                    Term::Access,
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
                        stm: vec![Statement::Expression(Expression::from(arr![
                            Term::Identifier(val),
                            Term::Assign(AssignKind::Normal),
                            Term::Deref,
                            Term::Identifier(nullable),
                        ]))],
                        ..Default::default()
                    }),
                })
            }
            "while" => Some(Statement::Conditional {
                cond: vec![(
                    Expression::from(arr![Term::Neg, Term::Group(self.exp(['{'], true)?.0)]),
                    self._break(),
                )],
                default: None,
            }),
            _ => None,
        } {
            stm.push(v)
        }

        Some(Statement::Loop(self._block(BlockType::Local, stm)?))
    }

    fn _break(&self) -> Block {
        Block {
            stm: vec![Statement::Break(Identifier::default())],
            ..Default::default()
        }
    }
}
