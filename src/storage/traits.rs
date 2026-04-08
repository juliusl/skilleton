//! SkillRepository trait and ItemRef/RepositoryError types (ADR-0008).

use std::fmt;

use super::StorageError;
use crate::types::*;

/// Storage interface for skill definitions.
pub trait SkillRepository {
    /// Load a skill by name (the slug from its ItemId).
    fn load_skill(&self, name: &str) -> Result<Skill, RepositoryError>;
    /// List available skill names.
    fn list_skills(&self) -> Result<Vec<String>, RepositoryError>;
    /// Find an item by its ItemId within a named skill.
    fn find_item(&self, skill: &str, id: &ItemId) -> Result<Option<ItemRef>, RepositoryError>;
}

/// Owned reference to a specific item kind.
#[derive(Debug, Clone, PartialEq)]
pub enum ItemRef {
    /// Reference to a Skill.
    Skill(Skill),
    /// Reference to a Procedure.
    Procedure(Procedure),
    /// Reference to a Step.
    Step(Step),
    /// Reference to a Task.
    Task(Task),
    /// Reference to a Policy.
    Policy(Policy),
    /// Reference to a Criterion.
    Criterion(Criterion),
}

/// Errors from repository operations.
#[derive(Debug)]
pub enum RepositoryError {
    /// Skill not found by name.
    NotFound(String),
    /// Storage-layer error (I/O, parse, etc.).
    Storage(StorageError),
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RepositoryError::NotFound(name) => write!(f, "skill not found: {name}"),
            RepositoryError::Storage(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for RepositoryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RepositoryError::Storage(e) => Some(e),
            RepositoryError::NotFound(_) => None,
        }
    }
}

impl From<StorageError> for RepositoryError {
    fn from(e: StorageError) -> Self {
        RepositoryError::Storage(e)
    }
}

/// Walk a Skill hierarchy to find an item by its ItemId.
pub(crate) fn find_item_in_skill(skill: &Skill, id: &ItemId) -> Option<ItemRef> {
    if skill.meta.id == *id {
        return Some(ItemRef::Skill(skill.clone()));
    }

    for policy in &skill.policies {
        if policy.meta.id == *id {
            return Some(ItemRef::Policy(policy.clone()));
        }
    }

    for criterion in &skill.criteria {
        if criterion.meta.id == *id {
            return Some(ItemRef::Criterion(criterion.clone()));
        }
    }

    for proc in &skill.procedures {
        if proc.meta.id == *id {
            return Some(ItemRef::Procedure(proc.clone()));
        }

        for policy in &proc.policies {
            if policy.meta.id == *id {
                return Some(ItemRef::Policy(policy.clone()));
            }
        }

        for criterion in &proc.criteria {
            if criterion.meta.id == *id {
                return Some(ItemRef::Criterion(criterion.clone()));
            }
        }

        for step in &proc.steps {
            if step.meta.id == *id {
                return Some(ItemRef::Step(step.clone()));
            }

            for policy in &step.policies {
                if policy.meta.id == *id {
                    return Some(ItemRef::Policy(policy.clone()));
                }
            }

            for criterion in &step.criteria {
                if criterion.meta.id == *id {
                    return Some(ItemRef::Criterion(criterion.clone()));
                }
            }

            for task in &step.tasks {
                if task.meta.id == *id {
                    return Some(ItemRef::Task(task.clone()));
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_ref_covers_all_types() {
        // Compile-time verification that ItemRef has all expected variants.
        let _: ItemRef = ItemRef::Skill(Skill {
            meta: ItemMeta { id: ItemId::parse("skill:test").unwrap(), conditions: vec![] },
            metadata: SkillMeta::default(),
            procedures: vec![],
            policies: vec![],
            criteria: vec![],
        });
        let _: ItemRef = ItemRef::Procedure(Procedure {
            meta: ItemMeta { id: ItemId::parse("procedure:test").unwrap(), conditions: vec![] },
            steps: vec![], entrance_criteria: vec![], exit_criteria: vec![], policies: vec![], criteria: vec![],
        });
        let _: ItemRef = ItemRef::Step(Step {
            meta: ItemMeta { id: ItemId::parse("step:test").unwrap(), conditions: vec![] },
            tasks: vec![], completion_criteria: vec![], policies: vec![], criteria: vec![],
        });
        let _: ItemRef = ItemRef::Task(Task {
            meta: ItemMeta { id: ItemId::parse("task:test").unwrap(), conditions: vec![] },
            subject: String::new(), action: String::new(), invokes: None,
        });
        let _: ItemRef = ItemRef::Policy(Policy {
            meta: ItemMeta { id: ItemId::parse("policy:test").unwrap(), conditions: vec![] },
            text: String::new(), compatible_with: vec![],
        });
        let _: ItemRef = ItemRef::Criterion(Criterion {
            meta: ItemMeta { id: ItemId::parse("criterion:test").unwrap(), conditions: vec![] },
            description: String::new(),
        });
    }

    #[test]
    fn repository_error_not_found_includes_name() {
        let err = RepositoryError::NotFound("my-skill".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("my-skill"));
    }

    #[test]
    fn repository_error_implements_error_trait() {
        let err = RepositoryError::NotFound("test".to_string());
        let _: &dyn std::error::Error = &err;
    }
}
