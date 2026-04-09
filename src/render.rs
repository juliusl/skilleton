//! Markdown rendering pipeline for skill build output (ADR-0010, ADR-0011).
//!
//! Walks the Skill tree depth-first, emitting Markdown with policy-first
//! ordering at every hierarchy level. Uses Mustache templates via ramhorns.

use ramhorns::{Content, Template};

use crate::types::{Criterion, CriterionRef, ItemMeta, Policy, Skill, Step, Task, Procedure};

/// The default Mustache template for rendering skills as Markdown.
/// Reproduces the same output as the original hardcoded `render_skill()`.
///
/// Note: ramhorns does not implement Mustache's standalone tag spec,
/// so section tags must be inline with content to avoid extra blank lines.
pub const DEFAULT_TEMPLATE: &str = "# {{{name}}}\
{{#has_description}}\n\n{{{description}}}\
{{/has_description}}\
{{#has_conditions}}\n*Conditions: {{{conditions}}}*\
{{/has_conditions}}\
{{#has_policies}}\n\n## Policies\n\
{{#policies}}\n> **{{{id}}}**: {{{text}}}\
{{/policies}}\
{{/has_policies}}\
{{#has_criteria}}\n\n## Criteria\n\
{{#criteria}}\n- **{{{id}}}**: {{{description}}}\
{{/criteria}}\
{{/has_criteria}}\
{{#has_procedures}}\n\n## Procedures\
{{#procedures}}\n\n### {{{id}}} — Procedure\
{{#has_conditions}}\n*Conditions: {{{conditions}}}*\
{{/has_conditions}}\
{{#has_policies}}\n\n**Policies:**\n\
{{#policies}}\n> **{{{id}}}**: {{{text}}}\
{{/policies}}\
{{/has_policies}}\
{{#has_criteria}}\n\n**Criteria:**\n\
{{#criteria}}\n- **{{{id}}}**: {{{description}}}\
{{/criteria}}\
{{/has_criteria}}\
{{#has_entrance_criteria}}\n\n**Entrance Criteria:**\n\
{{#entrance_criteria}}\n- {{{id}}}\
{{/entrance_criteria}}\
{{/has_entrance_criteria}}\
{{#steps}}\n\n#### {{{id}}} — Step\
{{#has_conditions}}\n*Conditions: {{{conditions}}}*\
{{/has_conditions}}\
{{#has_policies}}\n\n**Policies:**\n\
{{#policies}}\n> **{{{id}}}**: {{{text}}}\
{{/policies}}\
{{/has_policies}}\
{{#has_criteria}}\n\n**Criteria:**\n\
{{#criteria}}\n- **{{{id}}}**: {{{description}}}\
{{/criteria}}\
{{/has_criteria}}\
{{#has_tasks}}\n\n**Tasks:**\n\
{{#tasks}}\n- `{{{id}}}` **{{{subject}}}**: {{{action}}}{{{invokes_annotation}}}\
{{/tasks}}\
{{/has_tasks}}\
{{#has_completion_criteria}}\n\n**Completion Criteria:**\n\
{{#completion_criteria}}\n- {{{id}}}\
{{/completion_criteria}}\
{{/has_completion_criteria}}\
{{/steps}}\
{{#has_exit_criteria}}\n\n**Exit Criteria:**\n\
{{#exit_criteria}}\n- {{{id}}}\
{{/exit_criteria}}\
{{/has_exit_criteria}}\
{{/procedures}}\
{{/has_procedures}}\n";

// --- Template context structs ---

#[derive(Content)]
struct RenderContext {
    name: String,
    description: String,
    has_description: bool,
    has_conditions: bool,
    conditions: String,
    has_policies: bool,
    policies: Vec<PolicyCtx>,
    has_criteria: bool,
    criteria: Vec<CriterionCtx>,
    has_procedures: bool,
    procedures: Vec<ProcedureCtx>,
}

#[derive(Content)]
struct PolicyCtx {
    id: String,
    text: String,
}

#[derive(Content)]
struct CriterionCtx {
    id: String,
    description: String,
}

#[derive(Content)]
struct CriterionRefCtx {
    id: String,
}

#[derive(Content)]
struct TaskCtx {
    id: String,
    subject: String,
    action: String,
    invokes_annotation: String,
}

#[derive(Content)]
struct StepCtx {
    id: String,
    has_conditions: bool,
    conditions: String,
    has_policies: bool,
    policies: Vec<PolicyCtx>,
    has_criteria: bool,
    criteria: Vec<CriterionCtx>,
    has_tasks: bool,
    tasks: Vec<TaskCtx>,
    has_completion_criteria: bool,
    completion_criteria: Vec<CriterionRefCtx>,
}

#[derive(Content)]
struct ProcedureCtx {
    id: String,
    has_conditions: bool,
    conditions: String,
    has_policies: bool,
    policies: Vec<PolicyCtx>,
    has_criteria: bool,
    criteria: Vec<CriterionCtx>,
    has_entrance_criteria: bool,
    entrance_criteria: Vec<CriterionRefCtx>,
    steps: Vec<StepCtx>,
    has_exit_criteria: bool,
    exit_criteria: Vec<CriterionRefCtx>,
}

// --- Conversion helpers ---

fn format_conditions(meta: &ItemMeta) -> String {
    meta.conditions.iter().map(|c| c.id().as_str().to_string()).collect::<Vec<_>>().join(", ")
}

fn policy_ctx(p: &Policy) -> PolicyCtx {
    PolicyCtx { id: p.meta.id.as_str().to_string(), text: p.text.clone() }
}

fn criterion_ctx(c: &Criterion) -> CriterionCtx {
    CriterionCtx { id: c.meta.id.as_str().to_string(), description: c.description.clone() }
}

fn criterion_ref_ctx(cr: &CriterionRef) -> CriterionRefCtx {
    CriterionRefCtx { id: cr.id().as_str().to_string() }
}

fn task_ctx(t: &Task) -> TaskCtx {
    let invokes_annotation = t.invokes.as_ref()
        .map(|id| format!(" (invokes: {})", id.as_str()))
        .unwrap_or_default();
    TaskCtx {
        id: t.meta.id.as_str().to_string(),
        subject: t.subject.clone(),
        action: t.action.clone(),
        invokes_annotation,
    }
}

fn step_ctx(s: &Step) -> StepCtx {
    StepCtx {
        id: s.meta.id.as_str().to_string(),
        has_conditions: !s.meta.conditions.is_empty(),
        conditions: format_conditions(&s.meta),
        has_policies: !s.policies.is_empty(),
        policies: s.policies.iter().map(policy_ctx).collect(),
        has_criteria: !s.criteria.is_empty(),
        criteria: s.criteria.iter().map(criterion_ctx).collect(),
        has_tasks: !s.tasks.is_empty(),
        tasks: s.tasks.iter().map(task_ctx).collect(),
        has_completion_criteria: !s.completion_criteria.is_empty(),
        completion_criteria: s.completion_criteria.iter().map(criterion_ref_ctx).collect(),
    }
}

fn procedure_ctx(p: &Procedure) -> ProcedureCtx {
    ProcedureCtx {
        id: p.meta.id.as_str().to_string(),
        has_conditions: !p.meta.conditions.is_empty(),
        conditions: format_conditions(&p.meta),
        has_policies: !p.policies.is_empty(),
        policies: p.policies.iter().map(policy_ctx).collect(),
        has_criteria: !p.criteria.is_empty(),
        criteria: p.criteria.iter().map(criterion_ctx).collect(),
        has_entrance_criteria: !p.entrance_criteria.is_empty(),
        entrance_criteria: p.entrance_criteria.iter().map(criterion_ref_ctx).collect(),
        steps: p.steps.iter().map(step_ctx).collect(),
        has_exit_criteria: !p.exit_criteria.is_empty(),
        exit_criteria: p.exit_criteria.iter().map(criterion_ref_ctx).collect(),
    }
}

impl From<&Skill> for RenderContext {
    fn from(skill: &Skill) -> Self {
        RenderContext {
            name: skill.metadata.name.clone(),
            description: skill.metadata.description.clone(),
            has_description: !skill.metadata.description.is_empty(),
            has_conditions: !skill.meta.conditions.is_empty(),
            conditions: format_conditions(&skill.meta),
            has_policies: !skill.policies.is_empty(),
            policies: skill.policies.iter().map(policy_ctx).collect(),
            has_criteria: !skill.criteria.is_empty(),
            criteria: skill.criteria.iter().map(criterion_ctx).collect(),
            has_procedures: !skill.procedures.is_empty(),
            procedures: skill.procedures.iter().map(procedure_ctx).collect(),
        }
    }
}

/// Render a Skill as Markdown with policy-first ordering using the default template.
pub fn render_skill(skill: &Skill) -> String {
    render_skill_with_template(skill, DEFAULT_TEMPLATE)
        .expect("default template must parse")
}

/// Render a Skill as Markdown using a custom Mustache template.
pub fn render_skill_with_template(skill: &Skill, template: &str) -> Result<String, String> {
    let tpl = Template::new(template).map_err(|e| format!("template parse error: {e}"))?;
    let ctx = RenderContext::from(skill);
    Ok(tpl.render(&ctx))
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

    #[test]
    fn heading_hierarchy_uses_correct_levels() {
        let skill = make_skill_with_policies_and_procedures();
        let md = render_skill(&skill);
        // Skill title: #
        assert!(md.contains("# Test Skill\n"));
        // Skill-level sections: ##
        assert!(md.contains("## Policies\n"));
        assert!(md.contains("## Procedures\n"));
        // Procedure: ###
        assert!(md.contains("### procedure:p1 — Procedure\n"));
        // Step: ####
        assert!(md.contains("#### step:s1 — Step\n"));
    }
}
