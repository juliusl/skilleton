//! Skill validation — reference integrity, type-prefix consistency, and criterion references.
//!
//! Validates that `Task.invokes` references point to existing Procedures
//! and that the reference graph forms a DAG (no cycles). Also validates
//! that `CriterionRef` instances reference criterion-prefixed `ItemId`s
//! and that each item's `ItemId` type-prefix matches its struct type.

use std::collections::{HashMap, HashSet};
use std::fmt;
use super::types::{ItemId, Skill, TypePrefix};

/// Errors found during reference validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceError {
    /// A Task references a Procedure that does not exist in the Skill.
    MissingProcedure {
        task_id: ItemId,
        referenced_id: ItemId,
    },
    /// The reference graph contains a cycle, violating the DAG constraint.
    CycleDetected {
        cycle: Vec<ItemId>,
    },
}

impl fmt::Display for ReferenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReferenceError::MissingProcedure { task_id, referenced_id } => {
                write!(f, "task {task_id} references nonexistent procedure {referenced_id}")
            }
            ReferenceError::CycleDetected { cycle } => {
                let names: Vec<String> = cycle.iter().map(|id| id.to_string()).collect();
                write!(f, "cycle detected: {}", names.join(" → "))
            }
        }
    }
}

impl std::error::Error for ReferenceError {}

/// Validate all cross-procedure invocation references in a Skill.
/// Checks that invoked Procedures exist and the reference graph is a DAG.
pub fn validate_invocation_references(skill: &Skill) -> Result<(), Vec<ReferenceError>> {
    let mut errors = Vec::new();

    // Collect all procedure IDs
    let procedure_ids: HashSet<&ItemId> = skill
        .procedures
        .iter()
        .map(|p| &p.meta.id)
        .collect();

    // Build adjacency list: procedure_id -> [referenced_procedure_ids]
    let mut graph: HashMap<&ItemId, Vec<&ItemId>> = HashMap::new();
    for proc in &skill.procedures {
        graph.entry(&proc.meta.id).or_default();
    }

    // Walk all tasks and check references
    for proc in &skill.procedures {
        for step in &proc.steps {
            for task in &step.tasks {
                if let Some(ref target) = task.invokes {
                    if !procedure_ids.contains(target) {
                        errors.push(ReferenceError::MissingProcedure {
                            task_id: task.meta.id.clone(),
                            referenced_id: target.clone(),
                        });
                    } else {
                        graph.entry(&proc.meta.id).or_default().push(target);
                    }
                }
            }
        }
    }

    // Cycle detection via DFS (three-color marking).
    // Reports the first cycle found — multiple independent cycles require
    // iterative fix-and-revalidate.
    if let Some(cycle) = detect_cycle(&graph) {
        errors.push(ReferenceError::CycleDetected {
            cycle: cycle.into_iter().cloned().collect(),
        });
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Detect cycles using DFS-based approach, returning the cycle path if found.
fn detect_cycle<'a>(graph: &HashMap<&'a ItemId, Vec<&'a ItemId>>) -> Option<Vec<&'a ItemId>> {
    #[derive(Clone, Copy, PartialEq)]
    enum State { Unvisited, InProgress, Done }

    let mut state: HashMap<&ItemId, State> = graph.keys().map(|k| (*k, State::Unvisited)).collect();
    let mut path: Vec<&ItemId> = Vec::new();

    fn dfs<'a>(
        node: &'a ItemId,
        graph: &HashMap<&'a ItemId, Vec<&'a ItemId>>,
        state: &mut HashMap<&'a ItemId, State>,
        path: &mut Vec<&'a ItemId>,
    ) -> Option<Vec<&'a ItemId>> {
        state.insert(node, State::InProgress);
        path.push(node);

        if let Some(neighbors) = graph.get(node) {
            for &neighbor in neighbors {
                match state.get(neighbor) {
                    Some(State::InProgress) => {
                        // Found a cycle — extract it
                        let cycle_start = path.iter().position(|&n| n == neighbor).unwrap();
                        let mut cycle: Vec<&ItemId> = path[cycle_start..].to_vec();
                        cycle.push(neighbor);
                        return Some(cycle);
                    }
                    Some(State::Unvisited) | None => {
                        if let Some(cycle) = dfs(neighbor, graph, state, path) {
                            return Some(cycle);
                        }
                    }
                    Some(State::Done) => {}
                }
            }
        }

        path.pop();
        state.insert(node, State::Done);
        None
    }

    let nodes: Vec<&ItemId> = graph.keys().copied().collect();
    for node in nodes {
        if state.get(node) == Some(&State::Unvisited) {
            if let Some(cycle) = dfs(node, graph, &mut state, &mut path) {
                return Some(cycle);
            }
        }
    }
    None
}

/// Errors found during semantic validation of a Skill.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// A CriterionRef contains an ItemId with a non-criterion type prefix.
    InvalidCriterionRef {
        /// Location where the invalid CriterionRef was found.
        context_id: ItemId,
        /// The invalid ItemId inside the CriterionRef.
        criterion_id: ItemId,
        /// The actual prefix found.
        actual_prefix: TypePrefix,
    },
    /// An item's ItemId type-prefix does not match the expected struct type.
    TypePrefixMismatch {
        /// The item's ItemId.
        item_id: ItemId,
        /// The prefix found on the ItemId.
        actual: TypePrefix,
        /// The prefix expected for this struct type.
        expected: TypePrefix,
    },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidCriterionRef {
                context_id,
                criterion_id,
                actual_prefix,
            } => {
                write!(
                    f,
                    "invalid CriterionRef in {context_id}: {criterion_id} has prefix '{actual_prefix}', expected 'criterion'"
                )
            }
            ValidationError::TypePrefixMismatch {
                item_id,
                actual,
                expected,
            } => {
                write!(
                    f,
                    "type-prefix mismatch: {item_id} has prefix '{actual}', expected '{expected}'"
                )
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validate that all `CriterionRef` instances in a Skill reference criterion-prefixed `ItemId`s.
pub fn validate_criterion_references(skill: &Skill) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // Check skill-level criterion definitions (meta.conditions)
    check_criterion_refs(&skill.meta.id, &skill.meta.conditions, &mut errors);

    for proc in &skill.procedures {
        check_criterion_refs(&proc.meta.id, &proc.meta.conditions, &mut errors);
        check_criterion_refs(&proc.meta.id, &proc.entrance_criteria, &mut errors);
        check_criterion_refs(&proc.meta.id, &proc.exit_criteria, &mut errors);

        for step in &proc.steps {
            check_criterion_refs(&step.meta.id, &step.meta.conditions, &mut errors);
            check_criterion_refs(&step.meta.id, &step.completion_criteria, &mut errors);

            for task in &step.tasks {
                check_criterion_refs(&task.meta.id, &task.meta.conditions, &mut errors);
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_criterion_refs(
    context_id: &ItemId,
    refs: &[super::types::CriterionRef],
    errors: &mut Vec<ValidationError>,
) {
    for cref in refs {
        let prefix = cref.id().type_prefix();
        if prefix != TypePrefix::Criterion {
            errors.push(ValidationError::InvalidCriterionRef {
                context_id: context_id.clone(),
                criterion_id: cref.id().clone(),
                actual_prefix: prefix,
            });
        }
    }
}

/// Validate that every item's `ItemId` type-prefix matches its struct type.
///
/// Checks: Skill → `skill:`, Procedure → `procedure:`, Step → `step:`,
/// Task → `task:`, Policy → `policy:`, Criterion → `criterion:`.
pub fn validate_type_prefixes(skill: &Skill) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    check_prefix(&skill.meta.id, TypePrefix::Skill, &mut errors);
    check_policy_prefixes(&skill.policies, &mut errors);
    check_criterion_prefixes(&skill.criteria, &mut errors);

    for proc in &skill.procedures {
        check_prefix(&proc.meta.id, TypePrefix::Procedure, &mut errors);
        check_policy_prefixes(&proc.policies, &mut errors);
        check_criterion_prefixes(&proc.criteria, &mut errors);

        for step in &proc.steps {
            check_prefix(&step.meta.id, TypePrefix::Step, &mut errors);
            check_policy_prefixes(&step.policies, &mut errors);
            check_criterion_prefixes(&step.criteria, &mut errors);

            for task in &step.tasks {
                check_prefix(&task.meta.id, TypePrefix::Task, &mut errors);
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_prefix(id: &ItemId, expected: TypePrefix, errors: &mut Vec<ValidationError>) {
    let actual = id.type_prefix();
    if actual != expected {
        errors.push(ValidationError::TypePrefixMismatch {
            item_id: id.clone(),
            actual,
            expected,
        });
    }
}

fn check_policy_prefixes(
    policies: &[super::types::Policy],
    errors: &mut Vec<ValidationError>,
) {
    for policy in policies {
        check_prefix(&policy.meta.id, TypePrefix::Policy, errors);
    }
}

fn check_criterion_prefixes(
    criteria: &[super::types::Criterion],
    errors: &mut Vec<ValidationError>,
) {
    for criterion in criteria {
        check_prefix(&criterion.meta.id, TypePrefix::Criterion, errors);
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
            criteria: vec![],
        }
    }

    fn make_procedure(id: &str, steps: Vec<Step>) -> Procedure {
        Procedure {
            meta: make_meta(id),
            steps,
            entrance_criteria: vec![],
            exit_criteria: vec![],
            policies: vec![],
            criteria: vec![],
        }
    }

    #[test]
    fn valid_dag_passes() {
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
            criteria: vec![],
        };
        assert!(validate_invocation_references(&skill).is_ok());
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
            criteria: vec![],
        };
        let errs = validate_invocation_references(&skill).unwrap_err();
        assert!(errs.iter().any(|e| matches!(e, ReferenceError::MissingProcedure { .. })));
    }

    #[test]
    fn direct_cycle_detected() {
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
            criteria: vec![],
        };
        let errs = validate_invocation_references(&skill).unwrap_err();
        assert!(errs.iter().any(|e| matches!(e, ReferenceError::CycleDetected { .. })));
    }

    #[test]
    fn indirect_cycle_detected() {
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
            criteria: vec![],
        };
        let errs = validate_invocation_references(&skill).unwrap_err();
        assert!(errs.iter().any(|e| matches!(e, ReferenceError::CycleDetected { .. })));
    }

    #[test]
    fn self_reference_detected() {
        let skill = Skill {
            meta: make_meta("skill:test"),
            metadata: SkillMeta::default(),
            procedures: vec![
                make_procedure("procedure:a", vec![
                    make_step("step:a1", vec![make_task("task:a1", Some("procedure:a"))]),
                ]),
            ],
            policies: vec![],
            criteria: vec![],
        };
        let errs = validate_invocation_references(&skill).unwrap_err();
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
            criteria: vec![],
        };
        assert!(validate_invocation_references(&skill).is_ok());
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
            criteria: vec![],
        };
        let errs = validate_invocation_references(&skill).unwrap_err();
        let msg = format!("{}", errs[0]);
        assert!(msg.contains("task:a1"));
        assert!(msg.contains("procedure:missing"));
    }

    // -- QA plan findings --

    #[test]
    fn diamond_dag_passes() {
        // A→B, A→C, B→D, C→D (valid DAG with shared descendant)
        let skill = Skill {
            meta: make_meta("skill:test"),
            metadata: SkillMeta::default(),
            procedures: vec![
                make_procedure("procedure:a", vec![
                    make_step("step:a1", vec![
                        make_task("task:a1", Some("procedure:b")),
                        make_task("task:a2", Some("procedure:c")),
                    ]),
                ]),
                make_procedure("procedure:b", vec![
                    make_step("step:b1", vec![make_task("task:b1", Some("procedure:d"))]),
                ]),
                make_procedure("procedure:c", vec![
                    make_step("step:c1", vec![make_task("task:c1", Some("procedure:d"))]),
                ]),
                make_procedure("procedure:d", vec![
                    make_step("step:d1", vec![make_task("task:d1", None)]),
                ]),
            ],
            policies: vec![],
            criteria: vec![],
        };
        assert!(validate_invocation_references(&skill).is_ok());
    }

    #[test]
    fn direct_cycle_path_contains_involved_nodes() {
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
            criteria: vec![],
        };
        let errs = validate_invocation_references(&skill).unwrap_err();
        let cycle = errs.iter().find_map(|e| match e {
            ReferenceError::CycleDetected { cycle } => Some(cycle),
            _ => None,
        }).expect("should have a cycle error");
        assert!(cycle.contains(&make_id("procedure:a")));
        assert!(cycle.contains(&make_id("procedure:b")));
    }

    // -- CriterionRef validation tests --

    #[test]
    fn criterion_ref_validation_passes_with_valid_refs() {
        let skill = Skill {
            meta: make_meta("skill:test"),
            metadata: SkillMeta::default(),
            procedures: vec![Procedure {
                meta: make_meta("procedure:p"),
                steps: vec![Step {
                    meta: make_meta("step:s"),
                    tasks: vec![],
                    completion_criteria: vec![
                        CriterionRef::new_unchecked(make_id("criterion:done")),
                    ],
                    policies: vec![],
                    criteria: vec![],
                }],
                entrance_criteria: vec![
                    CriterionRef::new_unchecked(make_id("criterion:ready")),
                ],
                exit_criteria: vec![
                    CriterionRef::new_unchecked(make_id("criterion:complete")),
                ],
                policies: vec![],
                criteria: vec![],
            }],
            policies: vec![],
            criteria: vec![],
        };
        assert!(validate_criterion_references(&skill).is_ok());
    }

    #[test]
    fn criterion_ref_validation_catches_non_criterion_prefix() {
        let skill = Skill {
            meta: ItemMeta {
                id: make_id("skill:test"),
                conditions: vec![
                    CriterionRef::new_unchecked(make_id("policy:oops")),
                ],
            },
            metadata: SkillMeta::default(),
            procedures: vec![],
            policies: vec![],
            criteria: vec![],
        };
        let errs = validate_criterion_references(&skill).unwrap_err();
        assert_eq!(errs.len(), 1);
        assert!(matches!(
            &errs[0],
            ValidationError::InvalidCriterionRef { actual_prefix: TypePrefix::Policy, .. }
        ));
    }

    #[test]
    fn criterion_ref_validation_error_message_includes_context() {
        let skill = Skill {
            meta: ItemMeta {
                id: make_id("skill:test"),
                conditions: vec![
                    CriterionRef::new_unchecked(make_id("task:wrong")),
                ],
            },
            metadata: SkillMeta::default(),
            procedures: vec![],
            policies: vec![],
            criteria: vec![],
        };
        let errs = validate_criterion_references(&skill).unwrap_err();
        let msg = format!("{}", errs[0]);
        assert!(msg.contains("skill:test"));
        assert!(msg.contains("task:wrong"));
        assert!(msg.contains("criterion"));
    }

    // -- Type-prefix enforcement tests --

    #[test]
    fn type_prefix_validation_passes_for_correct_prefixes() {
        let skill = Skill {
            meta: make_meta("skill:test"),
            metadata: SkillMeta::default(),
            procedures: vec![Procedure {
                meta: make_meta("procedure:p"),
                steps: vec![Step {
                    meta: make_meta("step:s"),
                    tasks: vec![Task {
                        meta: make_meta("task:t"),
                        subject: "test".into(),
                        action: "test".into(),
                        invokes: None,
                    }],
                    completion_criteria: vec![],
                    policies: vec![Policy {
                        meta: make_meta("policy:p"),
                        text: "rule".into(),
                        compatible_with: vec![],
                    }],
                    criteria: vec![Criterion {
                        meta: make_meta("criterion:c"),
                        description: "state".into(),
                    }],
                }],
                entrance_criteria: vec![],
                exit_criteria: vec![],
                policies: vec![],
                criteria: vec![],
            }],
            policies: vec![],
            criteria: vec![],
        };
        assert!(validate_type_prefixes(&skill).is_ok());
    }

    #[test]
    fn type_prefix_validation_catches_skill_with_wrong_prefix() {
        let skill = Skill {
            meta: make_meta("procedure:not-a-skill"),
            metadata: SkillMeta::default(),
            procedures: vec![],
            policies: vec![],
            criteria: vec![],
        };
        let errs = validate_type_prefixes(&skill).unwrap_err();
        assert!(errs.iter().any(|e| matches!(
            e,
            ValidationError::TypePrefixMismatch {
                expected: TypePrefix::Skill,
                actual: TypePrefix::Procedure,
                ..
            }
        )));
    }

    #[test]
    fn type_prefix_validation_catches_procedure_with_wrong_prefix() {
        let skill = Skill {
            meta: make_meta("skill:test"),
            metadata: SkillMeta::default(),
            procedures: vec![Procedure {
                meta: make_meta("task:not-a-procedure"),
                steps: vec![],
                entrance_criteria: vec![],
                exit_criteria: vec![],
                policies: vec![],
                criteria: vec![],
            }],
            policies: vec![],
            criteria: vec![],
        };
        let errs = validate_type_prefixes(&skill).unwrap_err();
        assert!(errs.iter().any(|e| matches!(
            e,
            ValidationError::TypePrefixMismatch {
                expected: TypePrefix::Procedure,
                actual: TypePrefix::Task,
                ..
            }
        )));
    }

    #[test]
    fn type_prefix_validation_catches_multiple_mismatches() {
        let skill = Skill {
            meta: make_meta("task:wrong-skill"),
            metadata: SkillMeta::default(),
            procedures: vec![Procedure {
                meta: make_meta("skill:wrong-proc"),
                steps: vec![],
                entrance_criteria: vec![],
                exit_criteria: vec![],
                policies: vec![],
                criteria: vec![],
            }],
            policies: vec![],
            criteria: vec![],
        };
        let errs = validate_type_prefixes(&skill).unwrap_err();
        assert_eq!(errs.len(), 2);
    }

    #[test]
    fn type_prefix_validation_catches_policy_with_wrong_prefix() {
        let skill = Skill {
            meta: make_meta("skill:test"),
            metadata: SkillMeta::default(),
            procedures: vec![],
            policies: vec![Policy {
                meta: make_meta("criterion:not-a-policy"),
                text: "rule".into(),
                compatible_with: vec![],
            }],
            criteria: vec![],
        };
        let errs = validate_type_prefixes(&skill).unwrap_err();
        assert!(errs.iter().any(|e| matches!(
            e,
            ValidationError::TypePrefixMismatch {
                expected: TypePrefix::Policy,
                actual: TypePrefix::Criterion,
                ..
            }
        )));
    }

    #[test]
    fn type_prefix_mismatch_error_message_includes_details() {
        let skill = Skill {
            meta: make_meta("procedure:wrong"),
            metadata: SkillMeta::default(),
            procedures: vec![],
            policies: vec![],
            criteria: vec![],
        };
        let errs = validate_type_prefixes(&skill).unwrap_err();
        let msg = format!("{}", errs[0]);
        assert!(msg.contains("procedure:wrong"));
        assert!(msg.contains("procedure"));
        assert!(msg.contains("skill"));
    }
}
