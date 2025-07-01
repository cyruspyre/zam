use indexmap::IndexMap;

use crate::{
    log::Point,
    misc::RefMut,
    parser::{Parser, span::ToSpan},
    zam::{Entity, block::insert, identifier::Identifier},
};

impl Parser {
    pub fn r#use(
        &mut self,
        ext: &mut IndexMap<Identifier, RefMut<Entity>>,
        dup: &mut IndexMap<&String, Vec<([usize; 2], Point, &'static str)>>,
    ) -> Option<bool> {
        let id = self.identifier(true, true)?;

        if self.expect(&["::", ";"])? == ";" {
            insert(ext, dup, id, RefMut::default());
            return Some(true);
        }

        let mut stack = vec![id];
        let mut flag = true;

        loop {
            let Some(base) = stack.last() else { break };
            let idx = match flag {
                true => {
                    flag = false;
                    self.expect(&['{'])?;
                    self.ensure_closed('}')?
                }
                _ => *self.de.front().unwrap(),
            };

            if self.idx == idx || self.might('}') {
                self.de.pop_front();
                self.might(',');
                stack.pop();
                continue;
            }

            let mut id = self.identifier(true, true)?;

            if id[0].data == "self" {
                if id.is_qualified() {
                    self.log.err("`self` cannot be qualified in `use`")?
                }
                id.pop();
            }

            let mut id = id.qualify(base);

            if self.might('}') {
                insert(ext, dup, id, RefMut::default());
                continue;
            }

            let tmp = self.expect(&["::", "as", ","])?;

            if tmp == "as" {
                id.push(String::new().span(self.log.rng));
                id.push(self.identifier(true, false)?.pop().unwrap());
                self.expect(&[',', '}'])?;
            } else if tmp == "::" {
                flag = true;
                stack.push(id.clone());
            }

            if tmp != "::" {
                insert(ext, dup, id, RefMut::default())
            }
        }

        self.expect(&[';'])?;

        Some(true)
    }
}
