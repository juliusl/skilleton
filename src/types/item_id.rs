/// Unique identifier for an item, structured as a hierarchical path.
/// Placeholder — will be refined by ADR-0003.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemId(pub String);

/// Reference to a Criterion item by its ItemId.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriterionRef(pub ItemId);

/// Placeholder for agentskills.io specification metadata.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SkillMeta {
    pub name: String,
    pub description: String,
}
