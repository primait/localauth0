pub trait IsEmpty {
    fn is_empty(&self) -> bool;
}

impl IsEmpty for Option<String> {
    fn is_empty(&self) -> bool {
        match &self {
            None => true,
            Some(string) => string.is_empty(),
        }
    }
}
