use crate::{
    analyzer::Analyzer,
    log::{Log, Point},
    zam::{
        Entity,
        expression::{Expression, term::Term},
        statement::Statement,
        typ::{Type, kind::TypeKind},
    },
};

impl Analyzer {
    pub fn fun(&mut self, id_rng: [usize; 2], val: &mut Entity) {
        let Entity::Function {
            arg, ret, block, ..
        } = val
        else {
            return;
        };

        for v in arg.values_mut() {
            self.qualify_type(&mut v.kind);
        }

        self.qualify_type(&mut ret.kind);

        let Some(block) = block else { return };

        self.block(block, Some(ret));

        if match block.stm.last() {
            Some(Statement::Expression(exp)) if !exp.data.is_empty() => {
                matches!(exp.data[0].data, Term::Return(_))
            }
            _ => false,
        } {
            return;
        }

        if let TypeKind::Unit = ret.kind.data {
            return block.stm.push(Statement::Expression(Expression::new(
                [Term::Return(Expression {
                    data: Vec::new(),
                    typ: Type::unit([0; 2]),
                })],
                [0; 2],
            )));
        }

        self.cur().log.call(
            &mut [
                (
                    id_rng,
                    Point::Info,
                    "implicitly returns `()` as its body has no `return` expression at the end",
                ),
                (ret.kind.rng, Point::Info, "return type specified here"),
            ],
            Log::Error,
            "type mismatch",
            "",
        );
    }
}
