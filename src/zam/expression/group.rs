use super::Parser;

pub trait GroupValue {
    fn group_value(src: &mut Parser) -> Option<Option<Self>>
    where
        Self: Sized;
}

impl Parser {
    pub fn group<T: GroupValue>(&mut self) -> Option<Vec<T>> {
        self.expect(&['('])?;
        self.ensure_closed(')')?;

        let mut buf = Vec::new();

        while let Some(v) = T::group_value(self) {
            buf.push(v?);
            self.might(',');
        }

        self.idx += 1;
        self.de.pop_back();
        self.rng.fill(self.idx);

        Some(buf)
    }
}
