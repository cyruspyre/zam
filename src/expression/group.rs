use crate::source::Source;

pub trait GroupValue {
    fn group_value(src: &mut Source) -> Option<Self>
    where
        Self: Sized;
}

impl Source {
    pub fn group<T: GroupValue>(&mut self) -> Vec<T> {
        self.idx += 1;
        self.ensure_closed(')');

        let mut buf = Vec::new();

        while let Some(v) = T::group_value(self) {
            buf.push(v);
            self.might(',');
        }

        self.idx += 1;
        self.de.pop_back();

        buf
    }
}
