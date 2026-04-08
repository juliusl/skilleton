//! Singleton item types: Policy and Criterion.

use serde::{Deserialize, Serialize};

use super::{ItemId, ItemMeta};

/// A constraint or rule that MUST be followed. Singleton type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Policy {
    #[serde(flatten)]
    pub meta: ItemMeta,
    pub text: String,
    /// Policy IDs that this policy is known-compatible with.
    /// Mutual annotations suppress overlap detection (ADR-0007 §4).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub compatible_with: Vec<ItemId>,
}

/// A state or outcome that is either satisfied or unsatisfied. Singleton type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Criterion {
    #[serde(flatten)]
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
            compatible_with: vec![],
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

    #[test]
    fn serde_policy_round_trips() {
        let policy = Policy {
            meta: make_meta("policy:no-plaintext"),
            text: "Never store passwords in plaintext".to_string(),
            compatible_with: vec![],
        };
        let toml_str = toml::to_string(&policy).unwrap();
        let deserialized: Policy = toml::from_str(&toml_str).unwrap();
        assert_eq!(policy, deserialized);
    }

    #[test]
    fn serde_policy_empty_compatible_with_omitted() {
        let policy = Policy {
            meta: make_meta("policy:no-plaintext"),
            text: "Never store passwords in plaintext".to_string(),
            compatible_with: vec![],
        };
        let toml_str = toml::to_string(&policy).unwrap();
        assert!(!toml_str.contains("compatible_with"));
    }

    #[test]
    fn serde_policy_with_compatible_with_round_trips() {
        let policy = Policy {
            meta: make_meta("policy:no-plaintext"),
            text: "Never store passwords in plaintext".to_string(),
            compatible_with: vec![make_id("policy:encrypt-tokens")],
        };
        let toml_str = toml::to_string(&policy).unwrap();
        assert!(toml_str.contains("compatible_with"));
        let deserialized: Policy = toml::from_str(&toml_str).unwrap();
        assert_eq!(policy, deserialized);
    }

    #[test]
    fn serde_criterion_round_trips() {
        let criterion = Criterion {
            meta: make_meta("criterion:token-valid"),
            description: "JWT token has not expired".to_string(),
        };
        let toml_str = toml::to_string(&criterion).unwrap();
        let deserialized: Criterion = toml::from_str(&toml_str).unwrap();
        assert_eq!(criterion, deserialized);
    }
}
