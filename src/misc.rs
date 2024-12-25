pub trait ValidID {
    fn is_id(&self) -> bool;
}

impl ValidID for char {
    fn is_id(&self) -> bool {
        *self == '_' || self.is_ascii_alphanumeric()
    }
}
