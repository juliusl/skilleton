use super::{ItemId, CriterionRef};

/// Common metadata shared by all item types.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemMeta {
    pub id: ItemId,
    /// Conditional criteria — if empty, the item is implicitly Active.
    pub conditions: Vec<CriterionRef>,
}
