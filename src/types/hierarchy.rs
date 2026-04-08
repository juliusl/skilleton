use super::{ItemMeta, CriterionRef, Policy, SkillMeta, ItemId};

/// A single instruction with a subject and action. Hierarchy type.
#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub meta: ItemMeta,
    pub subject: String,
    pub action: String,
    /// Optional reference to another Procedure for composition (ADR-0004).
    pub invokes: Option<ItemId>,
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
        };
        let procedure = Procedure {
            meta: make_meta("procedure:onboard"),
            steps: vec![step],
            entrance_criteria: vec![],
            exit_criteria: vec![CriterionRef(make_id("criterion:onboarded"))],
            policies: vec![],
        };
        let skill = Skill {
            meta: make_meta("skill:onboarding"),
            metadata: SkillMeta::default(),
            procedures: vec![procedure],
            policies: vec![],
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
            }],
            policies: vec![Policy {
                meta: make_meta("policy:cloned"),
                text: "Must clone correctly".to_string(),
            }],
        };
        let cloned = skill.clone();
        assert_eq!(skill, cloned);
    }

    #[test]
    fn attach_policies_to_step() {
        let policy = Policy {
            meta: make_meta("policy:step-level"),
            text: "Step-level constraint".to_string(),
        };
        let step = Step {
            meta: make_meta("step:secured"),
            tasks: vec![],
            completion_criteria: vec![],
            policies: vec![policy],
        };
        assert_eq!(step.policies.len(), 1);
        assert_eq!(step.policies[0].text, "Step-level constraint");
    }

    #[test]
    fn attach_policies_to_procedure() {
        let policy = Policy {
            meta: make_meta("policy:proc-level"),
            text: "Procedure-level constraint".to_string(),
        };
        let procedure = Procedure {
            meta: make_meta("procedure:secured"),
            steps: vec![],
            entrance_criteria: vec![],
            exit_criteria: vec![],
            policies: vec![policy],
        };
        assert_eq!(procedure.policies.len(), 1);
    }

    #[test]
    fn attach_policies_to_skill() {
        let policy = Policy {
            meta: make_meta("policy:skill-level"),
            text: "Skill-level constraint".to_string(),
        };
        let skill = Skill {
            meta: make_meta("skill:secured"),
            metadata: SkillMeta::default(),
            procedures: vec![],
            policies: vec![policy],
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
        };
        assert_eq!(procedure.entrance_criteria.len(), 1);
        assert_eq!(procedure.exit_criteria.len(), 1);
    }

    #[test]
    fn independent_policy_sets_at_each_level() {
        let skill_policy = Policy {
            meta: make_meta("policy:global"),
            text: "Global constraint".to_string(),
        };
        let proc_policy = Policy {
            meta: make_meta("policy:local"),
            text: "Procedure-local constraint".to_string(),
        };
        let procedure = Procedure {
            meta: make_meta("procedure:mixed"),
            steps: vec![],
            entrance_criteria: vec![],
            exit_criteria: vec![],
            policies: vec![proc_policy],
        };
        let skill = Skill {
            meta: make_meta("skill:layered"),
            metadata: SkillMeta::default(),
            procedures: vec![procedure],
            policies: vec![skill_policy],
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
                Policy {
                    meta: make_meta("policy:first"),
                    text: "First constraint".to_string(),
                },
                Policy {
                    meta: make_meta("policy:second"),
                    text: "Second constraint".to_string(),
                },
                Policy {
                    meta: make_meta("policy:third"),
                    text: "Third constraint".to_string(),
                },
            ],
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
        };
        assert_eq!(procedure.entrance_criteria.len(), 2);
        assert_eq!(procedure.exit_criteria.len(), 3);
    }
}
