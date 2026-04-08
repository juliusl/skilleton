//! Hierarchy item types: Task, Step, Procedure, and Skill.

use serde::{Deserialize, Serialize};

use super::{ItemMeta, CriterionRef, Criterion, Policy, SkillMeta, ItemId};

/// A single instruction with a subject and action. Hierarchy type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    #[serde(flatten)]
    pub meta: ItemMeta,
    pub subject: String,
    pub action: String,
    /// Optional reference to another Procedure for composition (ADR-0004).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invokes: Option<ItemId>,
}

/// A set of Tasks with completion Criteria. Hierarchy type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Step {
    #[serde(flatten)]
    pub meta: ItemMeta,
    pub tasks: Vec<Task>,
    pub completion_criteria: Vec<CriterionRef>,
    pub policies: Vec<Policy>,
    /// Criterion definitions at step level (ADR-0006 §6).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub criteria: Vec<Criterion>,
}

/// A list of Steps with entrance and exit Criteria. Hierarchy type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Procedure {
    #[serde(flatten)]
    pub meta: ItemMeta,
    pub steps: Vec<Step>,
    pub entrance_criteria: Vec<CriterionRef>,
    pub exit_criteria: Vec<CriterionRef>,
    pub policies: Vec<Policy>,
    /// Criterion definitions at procedure level (ADR-0006 §6).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub criteria: Vec<Criterion>,
}

/// Root item with agentskills.io metadata. Hierarchy type (root).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Skill {
    #[serde(flatten)]
    pub meta: ItemMeta,
    #[serde(flatten)]
    pub metadata: SkillMeta,
    pub procedures: Vec<Procedure>,
    pub policies: Vec<Policy>,
    /// Criterion definitions at skill level (ADR-0006 §2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub criteria: Vec<Criterion>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ItemId, CriterionRef, ItemMeta, SkillMeta};

    fn make_id(s: &str) -> ItemId {
        ItemId::parse(s).expect("test ID should be valid")
    }

    fn make_meta(s: &str) -> ItemMeta {
        ItemMeta { id: make_id(s), conditions: vec![] }
    }

    fn make_policy(id: &str, text: &str) -> Policy {
        Policy {
            meta: make_meta(id),
            text: text.to_string(),
            compatible_with: vec![],
        }
    }

    #[test]
    fn construct_task() {
        let task = Task {
            meta: make_meta("task:check-jwt"),
            subject: "JWT token".to_string(),
            action: "Validate signature and expiry".to_string(),
            invokes: None,
        };
        assert_eq!(task.meta.id, make_id("task:check-jwt"));
    }

    #[test]
    fn construct_step() {
        let step = Step {
            meta: make_meta("step:validate-token"),
            tasks: vec![Task {
                meta: make_meta("task:check-jwt"),
                subject: "JWT token".to_string(),
                action: "Validate signature".to_string(),
                invokes: None,
            }],
            completion_criteria: vec![],
            policies: vec![],
            criteria: vec![],
        };
        assert_eq!(step.tasks.len(), 1);
    }

    #[test]
    fn construct_procedure() {
        let procedure = Procedure {
            meta: make_meta("procedure:auth-flow"),
            steps: vec![],
            entrance_criteria: vec![],
            exit_criteria: vec![],
            policies: vec![],
            criteria: vec![],
        };
        assert_eq!(procedure.meta.id, make_id("procedure:auth-flow"));
    }

    #[test]
    fn construct_skill() {
        let skill = Skill {
            meta: make_meta("skill:my-skill"),
            metadata: SkillMeta {
                name: "My Skill".to_string(),
                description: "A test skill".to_string(),
            },
            procedures: vec![],
            policies: vec![],
            criteria: vec![],
        };
        assert_eq!(skill.metadata.name, "My Skill");
    }

    #[test]
    fn build_full_hierarchy() {
        let task = Task {
            meta: make_meta("task:greet"),
            subject: "User".to_string(),
            action: "Send greeting message".to_string(),
            invokes: None,
        };
        let step = Step {
            meta: make_meta("step:welcome"),
            tasks: vec![task],
            completion_criteria: vec![CriterionRef(make_id("criterion:greeted"))],
            policies: vec![],
            criteria: vec![],
        };
        let procedure = Procedure {
            meta: make_meta("procedure:onboard"),
            steps: vec![step],
            entrance_criteria: vec![],
            exit_criteria: vec![CriterionRef(make_id("criterion:onboarded"))],
            policies: vec![],
            criteria: vec![],
        };
        let skill = Skill {
            meta: make_meta("skill:onboarding"),
            metadata: SkillMeta::default(),
            procedures: vec![procedure],
            policies: vec![],
            criteria: vec![],
        };

        assert_eq!(skill.procedures.len(), 1);
        assert_eq!(skill.procedures[0].steps.len(), 1);
        assert_eq!(skill.procedures[0].steps[0].tasks.len(), 1);
        assert_eq!(
            skill.procedures[0].steps[0].tasks[0].meta.id,
            make_id("task:greet")
        );
    }

    #[test]
    fn empty_hierarchy_skill_with_zero_procedures() {
        let skill = Skill {
            meta: make_meta("skill:empty"),
            metadata: SkillMeta::default(),
            procedures: vec![],
            policies: vec![],
            criteria: vec![],
        };
        assert!(skill.procedures.is_empty());
    }

    #[test]
    fn empty_hierarchy_procedure_with_zero_steps() {
        let procedure = Procedure {
            meta: make_meta("procedure:empty"),
            steps: vec![],
            entrance_criteria: vec![],
            exit_criteria: vec![],
            policies: vec![],
            criteria: vec![],
        };
        assert!(procedure.steps.is_empty());
    }

    #[test]
    fn empty_hierarchy_step_with_zero_tasks() {
        let step = Step {
            meta: make_meta("step:empty"),
            tasks: vec![],
            completion_criteria: vec![],
            policies: vec![],
            criteria: vec![],
        };
        assert!(step.tasks.is_empty());
    }

    #[test]
    fn clone_round_trip_produces_equal_value() {
        let skill = Skill {
            meta: make_meta("skill:original"),
            metadata: SkillMeta {
                name: "Test".to_string(),
                description: "Clone test".to_string(),
            },
            procedures: vec![Procedure {
                meta: make_meta("procedure:p1"),
                steps: vec![],
                entrance_criteria: vec![],
                exit_criteria: vec![],
                policies: vec![],
                criteria: vec![],
            }],
            policies: vec![make_policy("policy:cloned", "Must clone correctly")],
            criteria: vec![],
        };
        let cloned = skill.clone();
        assert_eq!(skill, cloned);
    }

    #[test]
    fn attach_policies_to_step() {
        let step = Step {
            meta: make_meta("step:secured"),
            tasks: vec![],
            completion_criteria: vec![],
            policies: vec![make_policy("policy:step-level", "Step-level constraint")],
            criteria: vec![],
        };
        assert_eq!(step.policies.len(), 1);
        assert_eq!(step.policies[0].text, "Step-level constraint");
    }

    #[test]
    fn attach_policies_to_procedure() {
        let procedure = Procedure {
            meta: make_meta("procedure:secured"),
            steps: vec![],
            entrance_criteria: vec![],
            exit_criteria: vec![],
            policies: vec![make_policy("policy:proc-level", "Procedure-level constraint")],
            criteria: vec![],
        };
        assert_eq!(procedure.policies.len(), 1);
    }

    #[test]
    fn attach_policies_to_skill() {
        let skill = Skill {
            meta: make_meta("skill:secured"),
            metadata: SkillMeta::default(),
            procedures: vec![],
            policies: vec![make_policy("policy:skill-level", "Skill-level constraint")],
            criteria: vec![],
        };
        assert_eq!(skill.policies.len(), 1);
    }

    #[test]
    fn attach_completion_criteria_to_step() {
        let step = Step {
            meta: make_meta("step:with-criteria"),
            tasks: vec![],
            completion_criteria: vec![
                CriterionRef(make_id("criterion:done")),
                CriterionRef(make_id("criterion:verified")),
            ],
            policies: vec![],
            criteria: vec![],
        };
        assert_eq!(step.completion_criteria.len(), 2);
    }

    #[test]
    fn attach_entrance_exit_criteria_to_procedure() {
        let procedure = Procedure {
            meta: make_meta("procedure:gated"),
            steps: vec![],
            entrance_criteria: vec![CriterionRef(make_id("criterion:ready"))],
            exit_criteria: vec![CriterionRef(make_id("criterion:complete"))],
            policies: vec![],
            criteria: vec![],
        };
        assert_eq!(procedure.entrance_criteria.len(), 1);
        assert_eq!(procedure.exit_criteria.len(), 1);
    }

    #[test]
    fn independent_policy_sets_at_each_level() {
        let procedure = Procedure {
            meta: make_meta("procedure:mixed"),
            steps: vec![],
            entrance_criteria: vec![],
            exit_criteria: vec![],
            policies: vec![make_policy("policy:local", "Procedure-local constraint")],
            criteria: vec![],
        };
        let skill = Skill {
            meta: make_meta("skill:layered"),
            metadata: SkillMeta::default(),
            procedures: vec![procedure],
            policies: vec![make_policy("policy:global", "Global constraint")],
            criteria: vec![],
        };

        assert_eq!(skill.policies.len(), 1);
        assert_eq!(skill.policies[0].text, "Global constraint");
        assert_eq!(skill.procedures[0].policies.len(), 1);
        assert_eq!(
            skill.procedures[0].policies[0].text,
            "Procedure-local constraint"
        );
    }

    #[test]
    fn multiple_policies_at_same_level() {
        let step = Step {
            meta: make_meta("step:multi-policy"),
            tasks: vec![],
            completion_criteria: vec![],
            policies: vec![
                make_policy("policy:first", "First constraint"),
                make_policy("policy:second", "Second constraint"),
                make_policy("policy:third", "Third constraint"),
            ],
            criteria: vec![],
        };
        assert_eq!(step.policies.len(), 3);
    }

    #[test]
    fn multiple_criteria_at_same_level() {
        let procedure = Procedure {
            meta: make_meta("procedure:multi-criteria"),
            steps: vec![],
            entrance_criteria: vec![
                CriterionRef(make_id("criterion:a")),
                CriterionRef(make_id("criterion:b")),
            ],
            exit_criteria: vec![
                CriterionRef(make_id("criterion:x")),
                CriterionRef(make_id("criterion:y")),
                CriterionRef(make_id("criterion:z")),
            ],
            policies: vec![],
            criteria: vec![],
        };
        assert_eq!(procedure.entrance_criteria.len(), 2);
        assert_eq!(procedure.exit_criteria.len(), 3);
    }

    #[test]
    fn serde_task_round_trips_without_invokes() {
        let task = Task {
            meta: make_meta("task:greet"),
            subject: "User".to_string(),
            action: "Send greeting".to_string(),
            invokes: None,
        };
        let toml_str = toml::to_string(&task).unwrap();
        assert!(!toml_str.contains("invokes"));
        let deserialized: Task = toml::from_str(&toml_str).unwrap();
        assert_eq!(task, deserialized);
    }

    #[test]
    fn serde_task_round_trips_with_invokes() {
        let task = Task {
            meta: make_meta("task:audit"),
            subject: "System".to_string(),
            action: "Log event".to_string(),
            invokes: Some(make_id("procedure:audit-log")),
        };
        let toml_str = toml::to_string(&task).unwrap();
        assert!(toml_str.contains("invokes"));
        let deserialized: Task = toml::from_str(&toml_str).unwrap();
        assert_eq!(task, deserialized);
    }

    #[test]
    fn serde_step_round_trips() {
        let step = Step {
            meta: make_meta("step:greet"),
            tasks: vec![Task {
                meta: make_meta("task:send"),
                subject: "User".to_string(),
                action: "Send message".to_string(),
                invokes: None,
            }],
            completion_criteria: vec![CriterionRef(make_id("criterion:greeted"))],
            policies: vec![make_policy("policy:greet-by-name", "Address by name")],
            criteria: vec![Criterion {
                meta: make_meta("criterion:greeted"),
                description: "User has been greeted".to_string(),
            }],
        };
        let toml_str = toml::to_string(&step).unwrap();
        let deserialized: Step = toml::from_str(&toml_str).unwrap();
        assert_eq!(step, deserialized);
    }

    #[test]
    fn serde_procedure_round_trips() {
        let procedure = Procedure {
            meta: make_meta("procedure:welcome"),
            steps: vec![Step {
                meta: make_meta("step:greet"),
                tasks: vec![],
                completion_criteria: vec![],
                policies: vec![],
                criteria: vec![],
            }],
            entrance_criteria: vec![CriterionRef(make_id("criterion:registered"))],
            exit_criteria: vec![CriterionRef(make_id("criterion:onboarded"))],
            policies: vec![],
            criteria: vec![],
        };
        let toml_str = toml::to_string(&procedure).unwrap();
        let deserialized: Procedure = toml::from_str(&toml_str).unwrap();
        assert_eq!(procedure, deserialized);
    }

    #[test]
    fn serde_skill_round_trips() {
        let skill = Skill {
            meta: make_meta("skill:onboarding"),
            metadata: SkillMeta {
                name: "Onboarding".to_string(),
                description: "New user onboarding flow".to_string(),
            },
            procedures: vec![Procedure {
                meta: make_meta("procedure:welcome"),
                steps: vec![],
                entrance_criteria: vec![],
                exit_criteria: vec![],
                policies: vec![],
                criteria: vec![],
            }],
            policies: vec![make_policy("policy:no-plaintext", "Never store passwords in plaintext")],
            criteria: vec![Criterion {
                meta: make_meta("criterion:onboarded"),
                description: "User has completed onboarding".to_string(),
            }],
        };
        let toml_str = toml::to_string(&skill).unwrap();
        let deserialized: Skill = toml::from_str(&toml_str).unwrap();
        assert_eq!(skill, deserialized);
    }

    #[test]
    fn serde_malformed_toml_produces_error() {
        let result: Result<Task, _> = toml::from_str("not valid toml {{{}");
        assert!(result.is_err());
    }
}
