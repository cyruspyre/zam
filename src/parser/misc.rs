pub trait CharExt {
    fn is_id(&self) -> bool;
    fn is_quote(&self) -> bool;
}

impl CharExt for char {
    fn is_id(&self) -> bool {
        *self == '_' || self.is_ascii_alphanumeric()
    }

    fn is_quote(&self) -> bool {
        matches!(self, '"' | '\'')
    }
}
