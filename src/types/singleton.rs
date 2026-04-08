use super::ItemMeta;

/// A constraint or rule that MUST be followed. Singleton type.
#[derive(Debug, Clone, PartialEq)]
pub struct Policy {
    pub meta: ItemMeta,
    pub text: String,
}

/// A state or outcome that is either satisfied or unsatisfied. Singleton type.
#[derive(Debug, Clone, PartialEq)]
pub struct Criterion {
    pub meta: ItemMeta,
    pub description: String,
}
