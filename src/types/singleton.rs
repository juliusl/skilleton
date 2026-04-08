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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ItemId, ItemMeta};

    fn make_id(s: &str) -> ItemId {
        ItemId::parse(s).expect("test ID should be valid")
    }

    fn make_meta(s: &str) -> ItemMeta {
        ItemMeta { id: make_id(s), conditions: vec![] }
    }

    #[test]
    fn construct_policy() {
        let policy = Policy {
            meta: make_meta("policy:no-plaintext"),
            text: "Never store passwords in plaintext".to_string(),
        };
        assert_eq!(policy.meta.id, make_id("policy:no-plaintext"));
    }

    #[test]
    fn construct_criterion() {
        let criterion = Criterion {
            meta: make_meta("criterion:token-valid"),
            description: "JWT token has not expired".to_string(),
        };
        assert_eq!(criterion.meta.id, make_id("criterion:token-valid"));
    }
}
