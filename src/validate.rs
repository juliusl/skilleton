use std::collections::{HashMap, HashSet};
use std::fmt;
use super::types::{ItemId, Skill};

/// Errors found during reference validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceError {
    MissingProcedure {
        task_id: ItemId,
        referenced_id: ItemId,
    },
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

/// Validate all cross-procedure references in a Skill.
/// Checks that invoked Procedures exist and the reference graph is a DAG.
pub fn validate_references(skill: &Skill) -> Result<(), Vec<ReferenceError>> {
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

    // Cycle detection via DFS (three-color marking)
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
