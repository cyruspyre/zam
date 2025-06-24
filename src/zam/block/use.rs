use indexmap::IndexMap;

use crate::{
    misc::RefMut,
    parser::{span::ToSpan, Parser},
    zam::{identifier::Identifier, Entity},
};

impl Parser {
    pub fn r#use(&mut self, ext: &mut IndexMap<Identifier, RefMut<Entity>>) -> Option<bool> {
        let id = self.identifier(true, true)?;

        if self.expect(&["::", ";"])? == ";" {
            ext.insert(id, RefMut::default());
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

            let mut id = self.identifier(true, true)?.qualify(base);

            if id.leaf_name().data == "self" {
                id.pop();
            }

            if self.might('}') {
                ext.insert(id, RefMut::default());
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
                ext.insert(id, RefMut::default());
            }
        }

        self.expect(&[';'])?;

        Some(true)
    }
}
