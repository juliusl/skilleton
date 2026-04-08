//! InMemoryRepository — test-friendly in-memory skill storage (ADR-0008).

use std::collections::HashMap;

use super::traits::{find_item_in_skill, ItemRef, RepositoryError, SkillRepository};
use crate::types::{ItemId, Skill};

/// In-memory repository backed by a HashMap. For tests and programmatic use.
pub struct InMemoryRepository {
    skills: HashMap<String, Skill>,
}

impl InMemoryRepository {
    /// Create a new empty repository.
    pub fn new() -> Self {
        InMemoryRepository {
            skills: HashMap::new(),
        }
    }

    /// Add a skill to the repository.
    pub fn add_skill(&mut self, name: String, skill: Skill) {
        self.skills.insert(name, skill);
    }
}

impl Default for InMemoryRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillRepository for InMemoryRepository {
    fn load_skill(&self, name: &str) -> Result<Skill, RepositoryError> {
        self.skills
            .get(name)
            .cloned()
            .ok_or_else(|| RepositoryError::NotFound(name.to_string()))
    }

    fn list_skills(&self) -> Result<Vec<String>, RepositoryError> {
        let mut names: Vec<String> = self.skills.keys().cloned().collect();
        names.sort();
        Ok(names)
    }

    fn find_item(&self, skill: &str, id: &ItemId) -> Result<Option<ItemRef>, RepositoryError> {
        let skill_data = self.load_skill(skill)?;
        Ok(find_item_in_skill(&skill_data, id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn make_id(s: &str) -> ItemId {
        ItemId::parse(s).expect("test ID should be valid")
    }

    fn make_meta(s: &str) -> ItemMeta {
        ItemMeta { id: make_id(s), conditions: vec![] }
    }

    fn make_test_skill() -> Skill {
        Skill {
            meta: make_meta("skill:test"),
            metadata: SkillMeta {
                name: "Test".to_string(),
                description: "A test skill".to_string(),
            },
            procedures: vec![Procedure {
                meta: make_meta("procedure:auth"),
                steps: vec![Step {
                    meta: make_meta("step:validate"),
                    tasks: vec![Task {
                        meta: make_meta("task:check"),
                        subject: "Token".to_string(),
                        action: "Validate JWT".to_string(),
                        invokes: None,
                    }],
                    completion_criteria: vec![],
                    policies: vec![Policy {
                        meta: make_meta("policy:step-level"),
                        text: "Step policy".to_string(),
                        compatible_with: vec![],
                    }],
                    criteria: vec![Criterion {
                        meta: make_meta("criterion:validated"),
                        description: "Token validated".to_string(),
                    }],
                }],
                entrance_criteria: vec![],
                exit_criteria: vec![],
                policies: vec![],
                criteria: vec![],
            }],
            policies: vec![Policy {
                meta: make_meta("policy:global"),
                text: "Global policy".to_string(),
                compatible_with: vec![],
            }],
            criteria: vec![],
        }
    }

    #[test]
    fn load_skill_returns_correct_skill() {
        let mut repo = InMemoryRepository::new();
        repo.add_skill("test".to_string(), make_test_skill());
        let skill = repo.load_skill("test").unwrap();
        assert_eq!(skill.meta.id, make_id("skill:test"));
    }

    #[test]
    fn load_skill_not_found() {
        let repo = InMemoryRepository::new();
        let result = repo.load_skill("nonexistent");
        assert!(matches!(result, Err(RepositoryError::NotFound(_))));
    }

    #[test]
    fn list_skills_returns_all_names() {
        let mut repo = InMemoryRepository::new();
        repo.add_skill("alpha".to_string(), make_test_skill());
        repo.add_skill("beta".to_string(), make_test_skill());
        let names = repo.list_skills().unwrap();
        assert_eq!(names, vec!["alpha", "beta"]);
    }

    #[test]
    fn list_skills_empty_when_no_skills() {
        let repo = InMemoryRepository::new();
        let names = repo.list_skills().unwrap();
        assert!(names.is_empty());
    }

    #[test]
    fn find_item_returns_procedure() {
        let mut repo = InMemoryRepository::new();
        repo.add_skill("test".to_string(), make_test_skill());
        let item = repo.find_item("test", &make_id("procedure:auth")).unwrap();
        assert!(matches!(item, Some(ItemRef::Procedure(_))));
    }

    #[test]
    fn find_item_returns_step() {
        let mut repo = InMemoryRepository::new();
        repo.add_skill("test".to_string(), make_test_skill());
        let item = repo.find_item("test", &make_id("step:validate")).unwrap();
        assert!(matches!(item, Some(ItemRef::Step(_))));
    }

    #[test]
    fn find_item_returns_task() {
        let mut repo = InMemoryRepository::new();
        repo.add_skill("test".to_string(), make_test_skill());
        let item = repo.find_item("test", &make_id("task:check")).unwrap();
        assert!(matches!(item, Some(ItemRef::Task(_))));
    }

    #[test]
    fn find_item_returns_policy() {
        let mut repo = InMemoryRepository::new();
        repo.add_skill("test".to_string(), make_test_skill());
        let item = repo.find_item("test", &make_id("policy:global")).unwrap();
        assert!(matches!(item, Some(ItemRef::Policy(_))));
    }

    #[test]
    fn find_item_returns_criterion() {
        let mut repo = InMemoryRepository::new();
        repo.add_skill("test".to_string(), make_test_skill());
        let item = repo.find_item("test", &make_id("criterion:validated")).unwrap();
        assert!(matches!(item, Some(ItemRef::Criterion(_))));
    }

    #[test]
    fn find_item_nonexistent_returns_none() {
        let mut repo = InMemoryRepository::new();
        repo.add_skill("test".to_string(), make_test_skill());
        let item = repo.find_item("test", &make_id("task:nonexistent")).unwrap();
        assert!(item.is_none());
    }

    #[test]
    fn find_item_nonexistent_skill_returns_error() {
        let repo = InMemoryRepository::new();
        let result = repo.find_item("nonexistent", &make_id("task:any"));
        assert!(matches!(result, Err(RepositoryError::NotFound(_))));
    }
}
