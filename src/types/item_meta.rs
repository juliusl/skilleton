//! Common metadata shared by all item types.

use serde::{Deserialize, Serialize};

use super::{ItemId, CriterionRef};

/// Common metadata shared by all item types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemMeta {
    pub id: ItemId,
    /// Conditional criteria — if empty, the item is implicitly Active.
    pub conditions: Vec<CriterionRef>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_id(s: &str) -> ItemId {
        ItemId::parse(s).expect("test ID should be valid")
    }

    #[test]
    fn empty_conditions_implicitly_active() {
        let meta = ItemMeta {
            id: make_id("task:any"),
            conditions: vec![],
        };
        assert!(meta.conditions.is_empty());
    }

    #[test]
    fn non_empty_conditions_contains_refs() {
        let meta = ItemMeta {
            id: make_id("task:guarded"),
            conditions: vec![
                CriterionRef::new_unchecked(make_id("criterion:enabled")),
                CriterionRef::new_unchecked(make_id("criterion:authorized")),
            ],
        };
        assert_eq!(meta.conditions.len(), 2);
        assert_eq!(
            meta.conditions[0],
            CriterionRef::new_unchecked(make_id("criterion:enabled"))
        );
    }

    #[test]
    fn serde_item_meta_round_trips() {
        let meta = ItemMeta {
            id: make_id("task:guarded"),
            conditions: vec![
                CriterionRef::new_unchecked(make_id("criterion:enabled")),
            ],
        };
        let serialized = toml::to_string(&meta).unwrap();
        let deserialized: ItemMeta = toml::from_str(&serialized).unwrap();
        assert_eq!(meta, deserialized);
    }
}
