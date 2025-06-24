use std::borrow::Borrow;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct NaiveMap<K, V>(Vec<(K, V)>);

impl<K: PartialEq, V> NaiveMap<K, V> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn insert(&mut self, k: K, v: V) -> bool {
        let tmp = self.find(&k).is_none();

        if tmp {
            self.0.push((k, v))
        }

        tmp
    }

    pub fn get<Q: PartialEq>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
    {
        match self.find(k) {
            Some(i) => Some(&mut self.0[i].1),
            _ => None,
        }
    }

    pub fn get_or_default(&mut self, k: K) -> &mut V
    where
        V: Default,
    {
        match self.find(&k) {
            Some(i) => &mut self.0[i].1,
            _ => {
                self.0.push((k, V::default()));
                &mut self.0.last_mut().unwrap().1
            }
        }
    }

    pub fn remove(&mut self, k: &K) {
        if let Some(index) = self.find(k) {
            self.0.swap_remove(index);
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Option<(K, V)> {
        self.0.pop()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    fn find<Q: PartialEq>(&self, k: &Q) -> Option<usize>
    where
        K: Borrow<Q>,
    {
        self.0.iter().position(|v| v.0.borrow().eq(k))
    }
}
