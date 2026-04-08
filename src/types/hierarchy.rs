use super::{ItemMeta, CriterionRef, Policy, SkillMeta};

/// A single instruction with a subject and action. Hierarchy type.
#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub meta: ItemMeta,
    pub subject: String,
    pub action: String,
}

/// A set of Tasks with completion Criteria. Hierarchy type.
#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    pub meta: ItemMeta,
    pub tasks: Vec<Task>,
    pub completion_criteria: Vec<CriterionRef>,
    pub policies: Vec<Policy>,
}

/// A list of Steps with entrance and exit Criteria. Hierarchy type.
#[derive(Debug, Clone, PartialEq)]
pub struct Procedure {
    pub meta: ItemMeta,
    pub steps: Vec<Step>,
    pub entrance_criteria: Vec<CriterionRef>,
    pub exit_criteria: Vec<CriterionRef>,
    pub policies: Vec<Policy>,
}

/// Root item with agentskills.io metadata. Hierarchy type (root).
#[derive(Debug, Clone, PartialEq)]
pub struct Skill {
    pub meta: ItemMeta,
    pub metadata: SkillMeta,
    pub procedures: Vec<Procedure>,
    pub policies: Vec<Policy>,
}
