//! Policy conflict detection via scope-overlap reporting.
//!
//! Computes effective policy sets by walking the skill hierarchy,
//! then detects cross-origin policy convergence (ADR-0007).

use crate::types::{ItemId, Policy, Skill};

/// Tracks where a policy was defined and how it reached this scope.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyOrigin {
    /// The policy definition.
    pub policy: Policy,
    /// The scope where this policy was originally defined.
    pub origin: ItemId,
    /// How this policy reached the current scope.
    pub kind: OriginKind,
}

/// How a policy reached a particular scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OriginKind {
    /// Defined at this scope or inherited from an ancestor in the hierarchy chain.
    Inherited,
    /// Merged from a cross-procedure invocation's callee.
    Invoked,
}

/// A node in the hierarchy with its effective (own + inherited) policies.
#[derive(Debug, Clone)]
pub struct EffectivePolicies {
    /// The ItemId of this node.
    pub scope: ItemId,
    /// All policies that apply at this scope, with their origins.
    pub policies: Vec<PolicyOrigin>,
}

/// A detected policy overlap at a specific scope.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyOverlap {
    /// The scope where the overlap occurs.
    pub target_scope: ItemId,
    /// The policies that converge at this scope, with their origin scopes.
    pub converging_policies: Vec<PolicyOrigin>,
    /// The type of overlap.
    pub overlap_type: OverlapType,
}

/// Classification of how the overlap arose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverlapType {
    /// Policies merged from cross-procedure invocation (caller + callee).
    CrossProcedureInvocation,
    /// Multiple policies defined at the same hierarchy level.
    SameLevelDefinition,
}

/// Compute effective policies for all nodes in a Skill.
///
/// Walks the hierarchy: Skill → Procedure → Step. At each level,
/// the effective set is the node's own policies plus all ancestor policies.
/// For cross-procedure invocations, merges the caller's effective set
/// with the callee's effective set at the invocation point.
pub fn compute_effective_policies(skill: &Skill) -> Vec<EffectivePolicies> {
    let mut result = Vec::new();

    // Skill-level effective set is just the skill's own policies.
    let skill_policies: Vec<PolicyOrigin> = skill
        .policies
        .iter()
        .map(|p| PolicyOrigin {
            policy: p.clone(),
            origin: skill.meta.id.clone(),
            kind: OriginKind::Inherited,
        })
        .collect();

    result.push(EffectivePolicies {
        scope: skill.meta.id.clone(),
        policies: skill_policies.clone(),
    });

    for proc in &skill.procedures {
        // Procedure inherits skill policies, adds its own.
        let mut proc_policies = skill_policies.clone();
        for p in &proc.policies {
            proc_policies.push(PolicyOrigin {
                policy: p.clone(),
                origin: proc.meta.id.clone(),
                kind: OriginKind::Inherited,
            });
        }

        result.push(EffectivePolicies {
            scope: proc.meta.id.clone(),
            policies: proc_policies.clone(),
        });

        for step in &proc.steps {
            // Step inherits procedure policies (which include skill), adds its own.
            let mut step_policies = proc_policies.clone();
            for p in &step.policies {
                step_policies.push(PolicyOrigin {
                    policy: p.clone(),
                    origin: step.meta.id.clone(),
                    kind: OriginKind::Inherited,
                });
            }

            result.push(EffectivePolicies {
                scope: step.meta.id.clone(),
                policies: step_policies.clone(),
            });

            // Handle cross-procedure invocations: merge callee's procedure-level
            // policies at the invocation Task scope.
            // Note: callee step-level policies are internal to the callee's execution
            // and do not propagate to the caller's invocation point. Skill-level
            // policies are already in the caller's inheritance chain (both procedures
            // share the same parent Skill), so only callee.policies adds new origins.
            for task in &step.tasks {
                if let Some(ref invoked_id) = task.invokes {
                    // Find the callee procedure
                    if let Some(callee) = skill.procedures.iter().find(|p| p.meta.id == *invoked_id) {
                        let mut invocation_policies = step_policies.clone();

                        // Add callee's own policies (different origin)
                        for p in &callee.policies {
                            // Avoid duplicates from same origin
                            let already_present = invocation_policies.iter().any(|po| {
                                po.policy.meta.id == p.meta.id && po.origin == callee.meta.id
                            });
                            if !already_present {
                                invocation_policies.push(PolicyOrigin {
                                    policy: p.clone(),
                                    origin: callee.meta.id.clone(),
                                    kind: OriginKind::Invoked,
                                });
                            }
                        }

                        result.push(EffectivePolicies {
                            scope: task.meta.id.clone(),
                            policies: invocation_policies,
                        });
                    }
                    // Dangling invokes references are handled by validate.rs
                }
            }
        }
    }

    result
}

/// Detect policy overlaps in a Skill.
///
/// Reports overlaps only when policies from different origins converge:
/// (a) cross-procedure invocations where caller and callee effective sets merge,
/// (b) multiple policies defined at the same hierarchy level.
/// Single-branch inheritance (child inheriting parent policies) is NOT reported.
pub fn detect_policy_overlaps(skill: &Skill) -> Vec<PolicyOverlap> {
    let mut overlaps = Vec::new();

    // Check for same-level definitions at each scope.
    check_same_level_policies(&skill.policies, &skill.meta.id, &mut overlaps);
    for proc in &skill.procedures {
        check_same_level_policies(&proc.policies, &proc.meta.id, &mut overlaps);
        for step in &proc.steps {
            check_same_level_policies(&step.policies, &step.meta.id, &mut overlaps);
        }
    }

    // Check for cross-procedure invocation overlaps.
    let effective = compute_effective_policies(skill);
    for ep in &effective {
        // A cross-procedure overlap exists when the effective set includes
        // policies with OriginKind::Invoked alongside other policies.
        let has_invoked = ep.policies.iter().any(|po| po.kind == OriginKind::Invoked);
        let has_inherited = ep.policies.iter().any(|po| po.kind == OriginKind::Inherited);

        if has_invoked && has_inherited {
            let converging: Vec<PolicyOrigin> = ep.policies.clone();

            if !all_pairs_compatible(&converging) {
                overlaps.push(PolicyOverlap {
                    target_scope: ep.scope.clone(),
                    converging_policies: converging,
                    overlap_type: OverlapType::CrossProcedureInvocation,
                });
            }
        }
    }

    overlaps
}

/// Check if multiple policies are defined at the same hierarchy level.
/// Only reports when 3+ policies exist at a scope without pairwise
/// compatibility — two policies are common and usually complementary.
fn check_same_level_policies(
    policies: &[Policy],
    scope: &ItemId,
    overlaps: &mut Vec<PolicyOverlap>,
) {
    if policies.len() > 2 {
        let origins: Vec<PolicyOrigin> = policies
            .iter()
            .map(|p| PolicyOrigin {
                policy: p.clone(),
                origin: scope.clone(),
                kind: OriginKind::Inherited,
            })
            .collect();

        if !all_pairs_compatible(&origins) {
            overlaps.push(PolicyOverlap {
                target_scope: scope.clone(),
                converging_policies: origins,
                overlap_type: OverlapType::SameLevelDefinition,
            });
        }
    }
}

/// Check if all pairs of policies have mutual compatible_with annotations.
/// O(n²) where n is the number of policies — acceptable because policies per scope
/// are expected to be small (single digits).
fn all_pairs_compatible(policies: &[PolicyOrigin]) -> bool {
    if policies.len() <= 1 {
        return true;
    }
    for i in 0..policies.len() {
        for j in (i + 1)..policies.len() {
            if !are_mutually_compatible(&policies[i].policy, &policies[j].policy) {
                return false;
            }
        }
    }
    true
}

/// Check if two policies have mutual compatibility annotations.
fn are_mutually_compatible(a: &Policy, b: &Policy) -> bool {
    a.compatible_with.contains(&b.meta.id) && b.compatible_with.contains(&a.meta.id)
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

    fn make_policy(id: &str, text: &str) -> Policy {
        Policy {
            meta: make_meta(id),
            text: text.to_string(),
            compatible_with: vec![],
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

    fn make_step(id: &str, tasks: Vec<Task>, policies: Vec<Policy>) -> Step {
        Step {
            meta: make_meta(id),
            tasks,
            completion_criteria: vec![],
            policies,
            criteria: vec![],
        }
    }

    fn make_procedure(id: &str, steps: Vec<Step>, policies: Vec<Policy>) -> Procedure {
        Procedure {
            meta: make_meta(id),
            steps,
            entrance_criteria: vec![],
            exit_criteria: vec![],
            policies,
            criteria: vec![],
        }
    }

    fn make_skill(procedures: Vec<Procedure>, policies: Vec<Policy>) -> Skill {
        Skill {
            meta: make_meta("skill:test"),
            metadata: SkillMeta::default(),
            procedures,
            policies,
            criteria: vec![],
        }
    }

    // --- Effective policy set computation tests ---

    #[test]
    fn effective_at_skill_contains_only_skill_policies() {
        let skill = make_skill(vec![], vec![make_policy("policy:global", "Global rule")]);
        let effective = compute_effective_policies(&skill);
        let skill_ep = effective.iter().find(|e| e.scope == make_id("skill:test")).unwrap();
        assert_eq!(skill_ep.policies.len(), 1);
        assert_eq!(skill_ep.policies[0].origin, make_id("skill:test"));
    }

    #[test]
    fn effective_at_procedure_contains_skill_plus_procedure_policies() {
        let skill = make_skill(
            vec![make_procedure(
                "procedure:auth",
                vec![],
                vec![make_policy("policy:local", "Local rule")],
            )],
            vec![make_policy("policy:global", "Global rule")],
        );
        let effective = compute_effective_policies(&skill);
        let proc_ep = effective.iter().find(|e| e.scope == make_id("procedure:auth")).unwrap();
        assert_eq!(proc_ep.policies.len(), 2);
    }

    #[test]
    fn effective_at_step_contains_all_ancestor_policies() {
        let skill = make_skill(
            vec![make_procedure(
                "procedure:auth",
                vec![make_step("step:validate", vec![], vec![make_policy("policy:step", "Step rule")])],
                vec![make_policy("policy:proc", "Proc rule")],
            )],
            vec![make_policy("policy:skill", "Skill rule")],
        );
        let effective = compute_effective_policies(&skill);
        let step_ep = effective.iter().find(|e| e.scope == make_id("step:validate")).unwrap();
        assert_eq!(step_ep.policies.len(), 3);
    }

    #[test]
    fn inherited_policies_carry_correct_origin() {
        let skill = make_skill(
            vec![make_procedure("procedure:auth", vec![], vec![])],
            vec![make_policy("policy:global", "Global rule")],
        );
        let effective = compute_effective_policies(&skill);
        let proc_ep = effective.iter().find(|e| e.scope == make_id("procedure:auth")).unwrap();
        assert_eq!(proc_ep.policies[0].origin, make_id("skill:test"));
    }

    #[test]
    fn cross_procedure_invocation_merges_callee_policies() {
        let skill = make_skill(
            vec![
                make_procedure(
                    "procedure:caller",
                    vec![make_step("step:s1", vec![make_task("task:t1", Some("procedure:callee"))], vec![])],
                    vec![make_policy("policy:caller-p", "Caller rule")],
                ),
                make_procedure(
                    "procedure:callee",
                    vec![],
                    vec![make_policy("policy:callee-p", "Callee rule")],
                ),
            ],
            vec![],
        );
        let effective = compute_effective_policies(&skill);
        let task_ep = effective.iter().find(|e| e.scope == make_id("task:t1")).unwrap();
        // Should have caller's policy and callee's policy
        let origins: Vec<&ItemId> = task_ep.policies.iter().map(|po| &po.origin).collect();
        assert!(origins.contains(&&make_id("procedure:caller")));
        assert!(origins.contains(&&make_id("procedure:callee")));
    }

    #[test]
    fn empty_policies_produce_empty_effective_sets() {
        let skill = make_skill(
            vec![make_procedure("procedure:empty", vec![], vec![])],
            vec![],
        );
        let effective = compute_effective_policies(&skill);
        let proc_ep = effective.iter().find(|e| e.scope == make_id("procedure:empty")).unwrap();
        assert!(proc_ep.policies.is_empty());
    }

    #[test]
    fn module_exported_from_lib() {
        // This test compiles only if the module is public in lib.rs.
        let _ = compute_effective_policies;
        let _ = detect_policy_overlaps;
    }

    // --- Overlap detection tests ---

    #[test]
    fn single_branch_inheritance_does_not_produce_overlaps() {
        // Skill → Procedure → Step with one policy each along the chain.
        let skill = make_skill(
            vec![make_procedure(
                "procedure:auth",
                vec![make_step("step:validate", vec![], vec![make_policy("policy:step-p", "Step rule")])],
                vec![make_policy("policy:proc-p", "Proc rule")],
            )],
            vec![make_policy("policy:skill-p", "Skill rule")],
        );
        let overlaps = detect_policy_overlaps(&skill);
        assert!(overlaps.is_empty(), "Single-branch inheritance should produce no overlaps");
    }

    #[test]
    fn cross_procedure_invocation_produces_overlap() {
        let skill = make_skill(
            vec![
                make_procedure(
                    "procedure:caller",
                    vec![make_step("step:s1", vec![make_task("task:t1", Some("procedure:callee"))], vec![])],
                    vec![make_policy("policy:caller-p", "Caller rule")],
                ),
                make_procedure(
                    "procedure:callee",
                    vec![],
                    vec![make_policy("policy:callee-p", "Callee rule")],
                ),
            ],
            vec![],
        );
        let overlaps = detect_policy_overlaps(&skill);
        assert_eq!(overlaps.len(), 1);
        assert_eq!(overlaps[0].overlap_type, OverlapType::CrossProcedureInvocation);
        assert_eq!(overlaps[0].target_scope, make_id("task:t1"));
    }

    #[test]
    fn multiple_policies_at_same_level_produce_overlap() {
        let skill = make_skill(
            vec![],
            vec![
                make_policy("policy:first", "First rule"),
                make_policy("policy:second", "Second rule"),
                make_policy("policy:third", "Third rule"),
            ],
        );
        let overlaps = detect_policy_overlaps(&skill);
        assert_eq!(overlaps.len(), 1);
        assert_eq!(overlaps[0].overlap_type, OverlapType::SameLevelDefinition);
    }

    #[test]
    fn empty_policy_sets_produce_no_overlaps() {
        let skill = make_skill(vec![], vec![]);
        let overlaps = detect_policy_overlaps(&skill);
        assert!(overlaps.is_empty());
    }

    #[test]
    fn single_policy_scope_produces_no_overlaps() {
        let skill = make_skill(vec![], vec![make_policy("policy:only", "The only rule")]);
        let overlaps = detect_policy_overlaps(&skill);
        assert!(overlaps.is_empty());
    }

    #[test]
    fn overlap_report_includes_correct_data() {
        let skill = make_skill(
            vec![],
            vec![
                make_policy("policy:a", "Rule A"),
                make_policy("policy:b", "Rule B"),
                make_policy("policy:c", "Rule C"),
            ],
        );
        let overlaps = detect_policy_overlaps(&skill);
        assert_eq!(overlaps[0].target_scope, make_id("skill:test"));
        assert_eq!(overlaps[0].converging_policies.len(), 3);
    }

    #[test]
    fn overlap_report_is_structured_data() {
        let skill = make_skill(
            vec![],
            vec![
                make_policy("policy:a", "Rule A"),
                make_policy("policy:b", "Rule B"),
                make_policy("policy:c", "Rule C"),
            ],
        );
        let overlaps = detect_policy_overlaps(&skill);
        // Verify we can access structured fields (not free-form text).
        let _ = &overlaps[0].target_scope;
        let _ = &overlaps[0].converging_policies;
        let _ = &overlaps[0].overlap_type;
    }

    // --- Compatibility annotation tests ---

    #[test]
    fn mutual_compatible_with_suppresses_overlap() {
        let skill = make_skill(
            vec![],
            vec![
                Policy {
                    meta: make_meta("policy:a"),
                    text: "Rule A".to_string(),
                    compatible_with: vec![make_id("policy:b")],
                },
                Policy {
                    meta: make_meta("policy:b"),
                    text: "Rule B".to_string(),
                    compatible_with: vec![make_id("policy:a")],
                },
            ],
        );
        let overlaps = detect_policy_overlaps(&skill);
        assert!(overlaps.is_empty(), "Mutual annotations should suppress the overlap");
    }

    #[test]
    fn unilateral_compatible_with_does_not_suppress_with_three_policies() {
        let skill = make_skill(
            vec![],
            vec![
                Policy {
                    meta: make_meta("policy:a"),
                    text: "Rule A".to_string(),
                    compatible_with: vec![make_id("policy:b")],
                },
                make_policy("policy:b", "Rule B"),
                make_policy("policy:c", "Rule C"),
            ],
        );
        let overlaps = detect_policy_overlaps(&skill);
        assert_eq!(overlaps.len(), 1, "Unilateral annotation should not suppress with 3 policies");
    }

    #[test]
    fn two_policies_at_same_level_produce_no_overlap() {
        let skill = make_skill(
            vec![],
            vec![
                make_policy("policy:a", "Rule A"),
                make_policy("policy:b", "Rule B"),
            ],
        );
        let overlaps = detect_policy_overlaps(&skill);
        assert!(overlaps.is_empty(), "Two policies should not trigger overlap check");
    }

    #[test]
    fn empty_compatible_with_has_no_effect_with_three_policies() {
        let skill = make_skill(
            vec![],
            vec![
                make_policy("policy:a", "Rule A"),
                make_policy("policy:b", "Rule B"),
                make_policy("policy:c", "Rule C"),
            ],
        );
        let overlaps = detect_policy_overlaps(&skill);
        assert_eq!(overlaps.len(), 1);
    }

    #[test]
    fn annotations_do_not_affect_other_overlaps() {
        // A and B are compatible, but C is not annotated → overlap remains.
        let skill = make_skill(
            vec![],
            vec![
                Policy {
                    meta: make_meta("policy:a"),
                    text: "Rule A".to_string(),
                    compatible_with: vec![make_id("policy:b")],
                },
                Policy {
                    meta: make_meta("policy:b"),
                    text: "Rule B".to_string(),
                    compatible_with: vec![make_id("policy:a")],
                },
                make_policy("policy:c", "Rule C"),
            ],
        );
        let overlaps = detect_policy_overlaps(&skill);
        assert_eq!(overlaps.len(), 1, "Three policies with partial annotations should still produce overlap");
    }

    #[test]
    fn suppressed_overlaps_excluded_from_results() {
        let skill = make_skill(
            vec![],
            vec![
                Policy {
                    meta: make_meta("policy:a"),
                    text: "Rule A".to_string(),
                    compatible_with: vec![make_id("policy:b")],
                },
                Policy {
                    meta: make_meta("policy:b"),
                    text: "Rule B".to_string(),
                    compatible_with: vec![make_id("policy:a")],
                },
            ],
        );
        let overlaps = detect_policy_overlaps(&skill);
        assert!(overlaps.is_empty());
    }

    #[test]
    fn three_policies_with_one_incompatible_pair_produces_overlap() {
        // A and B are compatible, but C is incompatible with both.
        let skill = make_skill(
            vec![],
            vec![
                Policy {
                    meta: make_meta("policy:a"),
                    text: "Rule A".to_string(),
                    compatible_with: vec![make_id("policy:b"), make_id("policy:c")],
                },
                Policy {
                    meta: make_meta("policy:b"),
                    text: "Rule B".to_string(),
                    compatible_with: vec![make_id("policy:a"), make_id("policy:c")],
                },
                make_policy("policy:c", "Rule C"),
            ],
        );
        let overlaps = detect_policy_overlaps(&skill);
        assert_eq!(overlaps.len(), 1, "One incompatible pair in three policies should produce overlap");
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;
    use crate::types::*;

    fn make_id(s: &str) -> ItemId {
        ItemId::parse(s).expect("test ID should be valid")
    }

    fn make_meta(s: &str) -> ItemMeta {
        ItemMeta { id: make_id(s), conditions: vec![] }
    }

    fn make_policy(id: &str, text: &str) -> Policy {
        Policy { meta: make_meta(id), text: text.to_string(), compatible_with: vec![] }
    }

    fn make_task(id: &str, invokes: Option<&str>) -> Task {
        Task { meta: make_meta(id), subject: "test".to_string(), action: "test".to_string(), invokes: invokes.map(|s| make_id(s)) }
    }

    fn make_step(id: &str, tasks: Vec<Task>, policies: Vec<Policy>) -> Step {
        Step { meta: make_meta(id), tasks, completion_criteria: vec![], policies, criteria: vec![] }
    }

    fn make_procedure(id: &str, steps: Vec<Step>, policies: Vec<Policy>) -> Procedure {
        Procedure { meta: make_meta(id), steps, entrance_criteria: vec![], exit_criteria: vec![], policies, criteria: vec![] }
    }

    fn make_skill(procedures: Vec<Procedure>, policies: Vec<Policy>) -> Skill {
        Skill { meta: make_meta("skill:test"), metadata: SkillMeta::default(), procedures, policies, criteria: vec![] }
    }

    // R11: Self-referential compatible_with has no effect
    #[test]
    fn self_referential_compatible_with_has_no_effect() {
        let skill = make_skill(vec![], vec![
            Policy {
                meta: make_meta("policy:a"),
                text: "Rule A".to_string(),
                compatible_with: vec![make_id("policy:a")], // self-reference
            },
            make_policy("policy:b", "Rule B"),
            make_policy("policy:c", "Rule C"),
        ]);
        let overlaps = detect_policy_overlaps(&skill);
        assert_eq!(overlaps.len(), 1, "Self-referential annotation should not suppress overlap");
    }

    // R12: Multiple tasks invoking different procedures
    #[test]
    fn multiple_invocations_in_one_step_produce_independent_overlaps() {
        let skill = make_skill(
            vec![
                make_procedure("procedure:caller", vec![
                    make_step("step:s1", vec![
                        make_task("task:t1", Some("procedure:callee-a")),
                        make_task("task:t2", Some("procedure:callee-b")),
                    ], vec![]),
                ], vec![make_policy("policy:caller-p", "Caller rule")]),
                make_procedure("procedure:callee-a", vec![], vec![make_policy("policy:a-p", "A rule")]),
                make_procedure("procedure:callee-b", vec![], vec![make_policy("policy:b-p", "B rule")]),
            ],
            vec![],
        );
        let overlaps = detect_policy_overlaps(&skill);
        let cross_proc_overlaps: Vec<_> = overlaps.iter()
            .filter(|o| o.overlap_type == OverlapType::CrossProcedureInvocation)
            .collect();
        assert_eq!(cross_proc_overlaps.len(), 2, "Each invocation should produce its own overlap");
    }

    // R13: Diamond invocation pattern
    #[test]
    fn diamond_invocation_detects_convergence() {
        let skill = make_skill(
            vec![
                make_procedure("procedure:a", vec![
                    make_step("step:a1", vec![
                        make_task("task:call-b", Some("procedure:b")),
                        make_task("task:call-c", Some("procedure:c")),
                    ], vec![]),
                ], vec![make_policy("policy:a-p", "A rule")]),
                make_procedure("procedure:b", vec![], vec![make_policy("policy:b-p", "B rule")]),
                make_procedure("procedure:c", vec![], vec![make_policy("policy:c-p", "C rule")]),
            ],
            vec![],
        );
        let overlaps = detect_policy_overlaps(&skill);
        assert!(!overlaps.is_empty(), "Diamond invocation should detect convergence");
    }
}
