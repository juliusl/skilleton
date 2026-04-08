//! Markdown rendering pipeline for skill build output (ADR-0010).
//!
//! Walks the Skill tree depth-first, emitting Markdown with policy-first
//! ordering at every hierarchy level.

use std::fmt::Write;

use crate::types::{Criterion, CriterionRef, ItemMeta, Policy, Skill, Step, Task, Procedure};

/// Render a Skill as Markdown with policy-first ordering.
pub fn render_skill(skill: &Skill) -> String {
    let mut out = String::new();

    // Skill title and description
    writeln!(out, "# {}", skill.metadata.name).unwrap();
    if !skill.metadata.description.is_empty() {
        writeln!(out).unwrap();
        writeln!(out, "{}", skill.metadata.description).unwrap();
    }

    render_conditions(&mut out, &skill.meta);

    // Policies (before procedures)
    if !skill.policies.is_empty() {
        writeln!(out).unwrap();
        writeln!(out, "## Policies").unwrap();
        writeln!(out).unwrap();
        for policy in &skill.policies {
            render_policy(&mut out, policy);
        }
    }

    // Criteria
    if !skill.criteria.is_empty() {
        writeln!(out).unwrap();
        writeln!(out, "## Criteria").unwrap();
        writeln!(out).unwrap();
        for criterion in &skill.criteria {
            render_criterion(&mut out, criterion);
        }
    }

    // Procedures
    if !skill.procedures.is_empty() {
        writeln!(out).unwrap();
        writeln!(out, "## Procedures").unwrap();
        for procedure in &skill.procedures {
            writeln!(out).unwrap();
            render_procedure(&mut out, procedure);
        }
    }

    out
}

fn render_procedure(out: &mut String, procedure: &Procedure) {
    writeln!(out, "### {} — Procedure", procedure.meta.id.as_str()).unwrap();

    render_conditions(out, &procedure.meta);

    if !procedure.policies.is_empty() {
        writeln!(out).unwrap();
        writeln!(out, "**Policies:**").unwrap();
        writeln!(out).unwrap();
        for policy in &procedure.policies {
            render_policy(out, policy);
        }
    }

    if !procedure.criteria.is_empty() {
        writeln!(out).unwrap();
        writeln!(out, "**Criteria:**").unwrap();
        writeln!(out).unwrap();
        for criterion in &procedure.criteria {
            render_criterion(out, criterion);
        }
    }

    if !procedure.entrance_criteria.is_empty() {
        writeln!(out).unwrap();
        writeln!(out, "**Entrance Criteria:**").unwrap();
        writeln!(out).unwrap();
        for cr in &procedure.entrance_criteria {
            render_criterion_ref(out, cr);
        }
    }

    for step in &procedure.steps {
        writeln!(out).unwrap();
        render_step(out, step);
    }

    if !procedure.exit_criteria.is_empty() {
        writeln!(out).unwrap();
        writeln!(out, "**Exit Criteria:**").unwrap();
        writeln!(out).unwrap();
        for cr in &procedure.exit_criteria {
            render_criterion_ref(out, cr);
        }
    }
}

fn render_step(out: &mut String, step: &Step) {
    writeln!(out, "#### {} — Step", step.meta.id.as_str()).unwrap();

    render_conditions(out, &step.meta);

    if !step.policies.is_empty() {
        writeln!(out).unwrap();
        writeln!(out, "**Policies:**").unwrap();
        writeln!(out).unwrap();
        for policy in &step.policies {
            render_policy(out, policy);
        }
    }

    if !step.criteria.is_empty() {
        writeln!(out).unwrap();
        writeln!(out, "**Criteria:**").unwrap();
        writeln!(out).unwrap();
        for criterion in &step.criteria {
            render_criterion(out, criterion);
        }
    }

    if !step.tasks.is_empty() {
        writeln!(out).unwrap();
        writeln!(out, "**Tasks:**").unwrap();
        writeln!(out).unwrap();
        for task in &step.tasks {
            render_task(out, task);
        }
    }

    if !step.completion_criteria.is_empty() {
        writeln!(out).unwrap();
        writeln!(out, "**Completion Criteria:**").unwrap();
        writeln!(out).unwrap();
        for cr in &step.completion_criteria {
            render_criterion_ref(out, cr);
        }
    }
}

fn render_task(out: &mut String, task: &Task) {
    let invokes = task
        .invokes
        .as_ref()
        .map(|id| format!(" (invokes: {})", id.as_str()))
        .unwrap_or_default();
    writeln!(
        out,
        "- `{}` **{}**: {}{}",
        task.meta.id.as_str(),
        task.subject,
        task.action,
        invokes,
    )
    .unwrap();
}

fn render_policy(out: &mut String, policy: &Policy) {
    writeln!(out, "> **{}**: {}", policy.meta.id.as_str(), policy.text).unwrap();
}

fn render_criterion(out: &mut String, criterion: &Criterion) {
    writeln!(
        out,
        "- **{}**: {}",
        criterion.meta.id.as_str(),
        criterion.description
    )
    .unwrap();
}

fn render_criterion_ref(out: &mut String, cr: &CriterionRef) {
    writeln!(out, "- {}", cr.id().as_str()).unwrap();
}

fn render_conditions(out: &mut String, meta: &ItemMeta) {
    if !meta.conditions.is_empty() {
        let refs: Vec<&str> = meta.conditions.iter().map(|c| c.id().as_str()).collect();
        writeln!(out, "*Conditions: {}*", refs.join(", ")).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ItemId, ItemMeta, SkillMeta};

    fn make_id(s: &str) -> ItemId {
        ItemId::parse(s).expect("test ID should be valid")
    }

    fn make_meta(s: &str) -> ItemMeta {
        ItemMeta {
            id: make_id(s),
            conditions: vec![],
        }
    }

    fn make_policy(id: &str, text: &str) -> Policy {
        Policy {
            meta: make_meta(id),
            text: text.to_string(),
            compatible_with: vec![],
        }
    }

    fn make_criterion(id: &str, desc: &str) -> Criterion {
        Criterion {
            meta: make_meta(id),
            description: desc.to_string(),
        }
    }

    fn make_skill_with_policies_and_procedures() -> Skill {
        Skill {
            meta: make_meta("skill:test"),
            metadata: SkillMeta {
                name: "Test Skill".to_string(),
                description: "A test skill".to_string(),
            },
            procedures: vec![Procedure {
                meta: make_meta("procedure:p1"),
                steps: vec![Step {
                    meta: make_meta("step:s1"),
                    tasks: vec![Task {
                        meta: make_meta("task:t1"),
                        subject: "User".to_string(),
                        action: "Do thing".to_string(),
                        invokes: None,
                    }],
                    completion_criteria: vec![],
                    policies: vec![],
                    criteria: vec![],
                }],
                entrance_criteria: vec![],
                exit_criteria: vec![],
                policies: vec![make_policy("policy:proc-rule", "Procedure rule")],
                criteria: vec![],
            }],
            policies: vec![make_policy("policy:skill-rule", "Skill rule")],
            criteria: vec![],
        }
    }

    #[test]
    fn policies_before_procedures_at_skill_level() {
        let skill = make_skill_with_policies_and_procedures();
        let md = render_skill(&skill);
        let policies_pos = md.find("## Policies").expect("should have Policies");
        let procedures_pos = md.find("## Procedures").expect("should have Procedures");
        assert!(
            policies_pos < procedures_pos,
            "Policies must appear before Procedures"
        );
    }

    #[test]
    fn policies_before_steps_at_procedure_level() {
        let skill = make_skill_with_policies_and_procedures();
        let md = render_skill(&skill);
        let policy_pos = md.find("**Policies:**").expect("should have procedure policies");
        let step_pos = md.find("#### step:s1 — Step").expect("should have step");
        assert!(
            policy_pos < step_pos,
            "Procedure policies must appear before steps"
        );
    }

    #[test]
    fn policies_before_tasks_at_step_level() {
        let skill = Skill {
            meta: make_meta("skill:test"),
            metadata: SkillMeta {
                name: "Test".to_string(),
                description: String::new(),
            },
            procedures: vec![Procedure {
                meta: make_meta("procedure:p1"),
                steps: vec![Step {
                    meta: make_meta("step:s1"),
                    tasks: vec![Task {
                        meta: make_meta("task:t1"),
                        subject: "User".to_string(),
                        action: "Do thing".to_string(),
                        invokes: None,
                    }],
                    completion_criteria: vec![],
                    policies: vec![make_policy("policy:step-rule", "Step rule")],
                    criteria: vec![],
                }],
                entrance_criteria: vec![],
                exit_criteria: vec![],
                policies: vec![],
                criteria: vec![],
            }],
            policies: vec![],
            criteria: vec![],
        };
        let md = render_skill(&skill);
        let policy_pos = md.find("**Policies:**").expect("should have step policies");
        let tasks_pos = md.find("**Tasks:**").expect("should have tasks");
        assert!(
            policy_pos < tasks_pos,
            "Step policies must appear before tasks"
        );
    }

    #[test]
    fn criterion_renders_with_description() {
        let skill = Skill {
            meta: make_meta("skill:test"),
            metadata: SkillMeta {
                name: "Test".to_string(),
                description: String::new(),
            },
            procedures: vec![],
            policies: vec![],
            criteria: vec![make_criterion("criterion:done", "Work is complete")],
        };
        let md = render_skill(&skill);
        assert!(md.contains("- **criterion:done**: Work is complete"));
    }

    #[test]
    fn criterion_ref_renders_id_only() {
        let skill = Skill {
            meta: make_meta("skill:test"),
            metadata: SkillMeta {
                name: "Test".to_string(),
                description: String::new(),
            },
            procedures: vec![Procedure {
                meta: make_meta("procedure:p1"),
                steps: vec![],
                entrance_criteria: vec![CriterionRef::new_unchecked(make_id("criterion:ready"))],
                exit_criteria: vec![],
                policies: vec![],
                criteria: vec![],
            }],
            policies: vec![],
            criteria: vec![],
        };
        let md = render_skill(&skill);
        assert!(md.contains("- criterion:ready"));
        // Should NOT contain description format for refs
        assert!(!md.contains("- **criterion:ready**:"));
    }

    #[test]
    fn conditions_rendered_as_italic_annotation() {
        let skill = Skill {
            meta: ItemMeta {
                id: make_id("skill:test"),
                conditions: vec![CriterionRef::new_unchecked(make_id("criterion:active"))],
            },
            metadata: SkillMeta {
                name: "Test".to_string(),
                description: String::new(),
            },
            procedures: vec![],
            policies: vec![],
            criteria: vec![],
        };
        let md = render_skill(&skill);
        assert!(md.contains("*Conditions: criterion:active*"));
    }

    #[test]
    fn empty_conditions_produce_no_output() {
        let skill = Skill {
            meta: make_meta("skill:test"),
            metadata: SkillMeta {
                name: "Test".to_string(),
                description: String::new(),
            },
            procedures: vec![],
            policies: vec![],
            criteria: vec![],
        };
        let md = render_skill(&skill);
        assert!(!md.contains("Conditions:"));
    }

    #[test]
    fn empty_sections_omitted() {
        let skill = Skill {
            meta: make_meta("skill:test"),
            metadata: SkillMeta {
                name: "Minimal".to_string(),
                description: String::new(),
            },
            procedures: vec![],
            policies: vec![],
            criteria: vec![],
        };
        let md = render_skill(&skill);
        assert!(!md.contains("## Policies"));
        assert!(!md.contains("## Criteria"));
        assert!(!md.contains("## Procedures"));
    }

    #[test]
    fn task_renders_id_subject_action_and_invokes() {
        let skill = Skill {
            meta: make_meta("skill:test"),
            metadata: SkillMeta {
                name: "Test".to_string(),
                description: String::new(),
            },
            procedures: vec![Procedure {
                meta: make_meta("procedure:p1"),
                steps: vec![Step {
                    meta: make_meta("step:s1"),
                    tasks: vec![
                        Task {
                            meta: make_meta("task:t1"),
                            subject: "System".to_string(),
                            action: "Log event".to_string(),
                            invokes: Some(make_id("procedure:audit")),
                        },
                        Task {
                            meta: make_meta("task:t2"),
                            subject: "User".to_string(),
                            action: "Greet".to_string(),
                            invokes: None,
                        },
                    ],
                    completion_criteria: vec![],
                    policies: vec![],
                    criteria: vec![],
                }],
                entrance_criteria: vec![],
                exit_criteria: vec![],
                policies: vec![],
                criteria: vec![],
            }],
            policies: vec![],
            criteria: vec![],
        };
        let md = render_skill(&skill);
        assert!(md.contains("- `task:t1` **System**: Log event (invokes: procedure:audit)"));
        assert!(md.contains("- `task:t2` **User**: Greet"));
        // No invokes annotation for t2
        assert!(!md.contains("task:t2` **User**: Greet (invokes:"));
    }

    #[test]
    fn policy_renders_as_blockquote() {
        let skill = Skill {
            meta: make_meta("skill:test"),
            metadata: SkillMeta {
                name: "Test".to_string(),
                description: String::new(),
            },
            procedures: vec![],
            policies: vec![make_policy("policy:p1", "Must follow this rule")],
            criteria: vec![],
        };
        let md = render_skill(&skill);
        assert!(md.contains("> **policy:p1**: Must follow this rule"));
    }

    #[test]
    fn definition_order_preserved() {
        let skill = Skill {
            meta: make_meta("skill:test"),
            metadata: SkillMeta {
                name: "Test".to_string(),
                description: String::new(),
            },
            procedures: vec![],
            policies: vec![
                make_policy("policy:first", "First"),
                make_policy("policy:second", "Second"),
                make_policy("policy:third", "Third"),
            ],
            criteria: vec![],
        };
        let md = render_skill(&skill);
        let first_pos = md.find("policy:first").unwrap();
        let second_pos = md.find("policy:second").unwrap();
        let third_pos = md.find("policy:third").unwrap();
        assert!(first_pos < second_pos);
        assert!(second_pos < third_pos);
    }
}
