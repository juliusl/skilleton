#[cfg(test)]
mod tests {
    use skilleton::types::*;

    fn make_id(s: &str) -> ItemId {
        ItemId::parse(s).expect("test ID should be valid")
    }

    fn make_meta(s: &str) -> ItemMeta {
        ItemMeta {
            id: make_id(s),
            conditions: vec![],
        }
    }

    // -- Task 2.1: Construction and composition tests --

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
    fn empty_conditions_implicitly_active() {
        let meta = make_meta("task:any");
        assert!(meta.conditions.is_empty());
    }

    #[test]
    fn non_empty_conditions_contains_refs() {
        let meta = ItemMeta {
            id: make_id("task:guarded"),
            conditions: vec![
                CriterionRef(make_id("criterion:enabled")),
                CriterionRef(make_id("criterion:authorized")),
            ],
        };
        assert_eq!(meta.conditions.len(), 2);
        assert_eq!(
            meta.conditions[0],
            CriterionRef(make_id("criterion:enabled"))
        );
    }

    // -- Task 2.2: Policy attachment and singleton tests --

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

    // -- QA findings: additional coverage --

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

    // -- ItemId parsing and validation tests (ADR-0003) --

    #[test]
    fn parse_valid_single_segment() {
        let id = ItemId::parse("skill:my-skill").unwrap();
        assert_eq!(id.to_string(), "skill:my-skill");
    }

    #[test]
    fn parse_valid_multi_segment() {
        let id = ItemId::parse("skill:my-skill.procedure:auth.step:validate").unwrap();
        assert_eq!(id.segments().len(), 3);
    }

    #[test]
    fn parse_rejects_invalid_prefix() {
        assert!(ItemId::parse("foo:bar").is_err());
    }

    #[test]
    fn parse_rejects_uppercase_slug() {
        assert!(ItemId::parse("skill:MySkill").is_err());
    }

    #[test]
    fn parse_rejects_slug_with_spaces() {
        assert!(ItemId::parse("skill:my skill").is_err());
    }

    #[test]
    fn parse_rejects_slug_over_50_chars() {
        let long_slug = "a".repeat(51);
        assert!(ItemId::parse(&format!("skill:{long_slug}")).is_err());
    }

    #[test]
    fn parse_rejects_empty_string() {
        assert!(ItemId::parse("").is_err());
    }

    #[test]
    fn parse_rejects_malformed_segment() {
        assert!(ItemId::parse("nocolon").is_err());
    }

    #[test]
    fn segments_returns_correct_parts() {
        let id = ItemId::parse("skill:s1.procedure:p1").unwrap();
        let segs = id.segments();
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].type_prefix, TypePrefix::Skill);
        assert_eq!(segs[0].slug, "s1");
        assert_eq!(segs[1].type_prefix, TypePrefix::Procedure);
        assert_eq!(segs[1].slug, "p1");
    }

    #[test]
    fn display_round_trips() {
        let original = "skill:s1.procedure:p1.step:s1";
        let id = ItemId::parse(original).unwrap();
        let reparsed = ItemId::parse(&id.to_string()).unwrap();
        assert_eq!(id, reparsed);
    }

    #[test]
    fn parent_returns_none_for_single_segment() {
        let id = ItemId::parse("skill:root").unwrap();
        assert!(id.parent().is_none());
    }

    #[test]
    fn parent_returns_parent_path() {
        let id = ItemId::parse("skill:s1.procedure:p1.step:s1").unwrap();
        let parent = id.parent().unwrap();
        assert_eq!(parent, ItemId::parse("skill:s1.procedure:p1").unwrap());
    }

    #[test]
    fn prefix_matches_self() {
        let id = ItemId::parse("skill:s1.procedure:p1").unwrap();
        assert!(id.prefix_matches(&id));
    }

    #[test]
    fn prefix_matches_ancestor() {
        let id = ItemId::parse("skill:s1.procedure:p1.step:s1").unwrap();
        let prefix = ItemId::parse("skill:s1").unwrap();
        assert!(id.prefix_matches(&prefix));
    }

    #[test]
    fn prefix_does_not_match_partial_segment() {
        let id = ItemId::parse("skill:s1-extra").unwrap();
        let prefix = ItemId::parse("skill:s1").unwrap();
        assert!(!id.prefix_matches(&prefix));
    }

    #[test]
    fn append_creates_child_path() {
        let parent = ItemId::parse("skill:s1").unwrap();
        let child = parent.append(TypePrefix::Procedure, "auth").unwrap();
        assert_eq!(child, ItemId::parse("skill:s1.procedure:auth").unwrap());
    }
}

#[cfg(test)]
mod validate_tests {
    use skilleton::types::*;
    use skilleton::validate::*;

    fn make_id(s: &str) -> ItemId {
        ItemId::parse(s).expect("test ID should be valid")
    }

    fn make_meta(s: &str) -> ItemMeta {
        ItemMeta {
            id: make_id(s),
            conditions: vec![],
        }
    }

    fn make_task(id: &str, invokes: Option<&str>) -> Task {
        Task {
            meta: make_meta(id),
            subject: "test".to_string(),
            action: "test".to_string(),
            invokes: invokes.map(|s| make_id(s)),
        }
    }

    fn make_step(id: &str, tasks: Vec<Task>) -> Step {
        Step {
            meta: make_meta(id),
            tasks,
            completion_criteria: vec![],
            policies: vec![],
        }
    }

    fn make_procedure(id: &str, steps: Vec<Step>) -> Procedure {
        Procedure {
            meta: make_meta(id),
            steps,
            entrance_criteria: vec![],
            exit_criteria: vec![],
            policies: vec![],
        }
    }

    #[test]
    fn valid_dag_passes() {
        // A -> B -> C (no cycles)
        let skill = Skill {
            meta: make_meta("skill:test"),
            metadata: SkillMeta::default(),
            procedures: vec![
                make_procedure("procedure:a", vec![
                    make_step("step:a1", vec![make_task("task:a1", Some("procedure:b"))]),
                ]),
                make_procedure("procedure:b", vec![
                    make_step("step:b1", vec![make_task("task:b1", Some("procedure:c"))]),
                ]),
                make_procedure("procedure:c", vec![
                    make_step("step:c1", vec![make_task("task:c1", None)]),
                ]),
            ],
            policies: vec![],
        };
        assert!(validate_references(&skill).is_ok());
    }

    #[test]
    fn missing_procedure_returns_error() {
        let skill = Skill {
            meta: make_meta("skill:test"),
            metadata: SkillMeta::default(),
            procedures: vec![
                make_procedure("procedure:a", vec![
                    make_step("step:a1", vec![
                        make_task("task:a1", Some("procedure:nonexistent")),
                    ]),
                ]),
            ],
            policies: vec![],
        };
        let errs = validate_references(&skill).unwrap_err();
        assert!(errs.iter().any(|e| matches!(e, ReferenceError::MissingProcedure { .. })));
    }

    #[test]
    fn direct_cycle_detected() {
        // A -> B -> A
        let skill = Skill {
            meta: make_meta("skill:test"),
            metadata: SkillMeta::default(),
            procedures: vec![
                make_procedure("procedure:a", vec![
                    make_step("step:a1", vec![make_task("task:a1", Some("procedure:b"))]),
                ]),
                make_procedure("procedure:b", vec![
                    make_step("step:b1", vec![make_task("task:b1", Some("procedure:a"))]),
                ]),
            ],
            policies: vec![],
        };
        let errs = validate_references(&skill).unwrap_err();
        assert!(errs.iter().any(|e| matches!(e, ReferenceError::CycleDetected { .. })));
    }

    #[test]
    fn indirect_cycle_detected() {
        // A -> B -> C -> A
        let skill = Skill {
            meta: make_meta("skill:test"),
            metadata: SkillMeta::default(),
            procedures: vec![
                make_procedure("procedure:a", vec![
                    make_step("step:a1", vec![make_task("task:a1", Some("procedure:b"))]),
                ]),
                make_procedure("procedure:b", vec![
                    make_step("step:b1", vec![make_task("task:b1", Some("procedure:c"))]),
                ]),
                make_procedure("procedure:c", vec![
                    make_step("step:c1", vec![make_task("task:c1", Some("procedure:a"))]),
                ]),
            ],
            policies: vec![],
        };
        let errs = validate_references(&skill).unwrap_err();
        assert!(errs.iter().any(|e| matches!(e, ReferenceError::CycleDetected { .. })));
    }

    #[test]
    fn self_reference_detected() {
        // A -> A
        let skill = Skill {
            meta: make_meta("skill:test"),
            metadata: SkillMeta::default(),
            procedures: vec![
                make_procedure("procedure:a", vec![
                    make_step("step:a1", vec![make_task("task:a1", Some("procedure:a"))]),
                ]),
            ],
            policies: vec![],
        };
        let errs = validate_references(&skill).unwrap_err();
        assert!(errs.iter().any(|e| matches!(e, ReferenceError::CycleDetected { .. })));
    }

    #[test]
    fn no_invokes_passes() {
        let skill = Skill {
            meta: make_meta("skill:test"),
            metadata: SkillMeta::default(),
            procedures: vec![
                make_procedure("procedure:a", vec![
                    make_step("step:a1", vec![make_task("task:a1", None)]),
                ]),
            ],
            policies: vec![],
        };
        assert!(validate_references(&skill).is_ok());
    }

    #[test]
    fn error_messages_include_ids() {
        let skill = Skill {
            meta: make_meta("skill:test"),
            metadata: SkillMeta::default(),
            procedures: vec![
                make_procedure("procedure:a", vec![
                    make_step("step:a1", vec![
                        make_task("task:a1", Some("procedure:missing")),
                    ]),
                ]),
            ],
            policies: vec![],
        };
        let errs = validate_references(&skill).unwrap_err();
        let msg = format!("{}", errs[0]);
        assert!(msg.contains("task:a1"));
        assert!(msg.contains("procedure:missing"));
    }
}
