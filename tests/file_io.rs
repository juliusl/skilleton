//! Integration tests for skill file I/O round-trips (ADR-0006).

use skilleton::types::*;
use skilleton::storage::{SkillWriter, SkillLoader};

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

fn build_full_skill() -> Skill {
    Skill {
        meta: make_meta("skill:onboarding"),
        metadata: SkillMeta {
            name: "Onboarding".to_string(),
            description: "New user onboarding flow".to_string(),
        },
        procedures: vec![
            Procedure {
                meta: make_meta("procedure:welcome"),
                steps: vec![Step {
                    meta: make_meta("step:greet"),
                    tasks: vec![
                        Task {
                            meta: make_meta("task:send-message"),
                            subject: "User".to_string(),
                            action: "Send greeting message".to_string(),
                            invokes: None,
                        },
                        Task {
                            meta: make_meta("task:log-event"),
                            subject: "System".to_string(),
                            action: "Log onboarding event".to_string(),
                            invokes: Some(make_id("procedure:audit-log")),
                        },
                    ],
                    completion_criteria: vec![CriterionRef(make_id("criterion:greeted"))],
                    policies: vec![make_policy("policy:greet-by-name", "Address user by name")],
                    criteria: vec![Criterion {
                        meta: make_meta("criterion:greeted"),
                        description: "User has been greeted".to_string(),
                    }],
                }],
                entrance_criteria: vec![CriterionRef(make_id("criterion:registered"))],
                exit_criteria: vec![CriterionRef(make_id("criterion:onboarded"))],
                policies: vec![],
                criteria: vec![],
            },
            Procedure {
                meta: make_meta("procedure:audit-log"),
                steps: vec![],
                entrance_criteria: vec![],
                exit_criteria: vec![],
                policies: vec![make_policy("policy:audit-required", "All events must be logged")],
                criteria: vec![],
            },
        ],
        policies: vec![make_policy("policy:no-plaintext", "Never store passwords in plaintext")],
        criteria: vec![Criterion {
            meta: make_meta("criterion:onboarded"),
            description: "User has completed onboarding".to_string(),
        }],
    }
}

#[test]
fn round_trip_write_then_load() {
    let dir = tempfile::tempdir().unwrap();
    let skill = build_full_skill();
    SkillWriter::write(dir.path(), &skill).unwrap();

    let mut loaded = SkillLoader::load(&dir.path().join("onboarding")).unwrap();

    // SkillLoader returns procedures sorted alphabetically by filename.
    // Sort the original skill's procedures the same way for comparison.
    let mut expected = skill;
    expected.procedures.sort_by(|a, b| {
        a.meta.id.as_str().cmp(b.meta.id.as_str())
    });
    loaded.procedures.sort_by(|a, b| {
        a.meta.id.as_str().cmp(b.meta.id.as_str())
    });

    assert_eq!(expected, loaded);
}

#[test]
fn round_trip_with_cross_procedure_invokes() {
    let dir = tempfile::tempdir().unwrap();
    let skill = build_full_skill();
    SkillWriter::write(dir.path(), &skill).unwrap();

    let loaded = SkillLoader::load(&dir.path().join("onboarding")).unwrap();

    // Find the welcome procedure's task that invokes audit-log
    let welcome = loaded.procedures.iter().find(|p| {
        p.meta.id == make_id("procedure:welcome")
    }).expect("welcome procedure should exist");

    let log_task = &welcome.steps[0].tasks.iter().find(|t| {
        t.meta.id == make_id("task:log-event")
    }).expect("log-event task should exist");

    assert_eq!(log_task.invokes, Some(make_id("procedure:audit-log")));
}

#[test]
fn round_trip_with_policies_at_all_levels() {
    let dir = tempfile::tempdir().unwrap();
    let skill = build_full_skill();
    SkillWriter::write(dir.path(), &skill).unwrap();

    let loaded = SkillLoader::load(&dir.path().join("onboarding")).unwrap();

    // Skill-level policy
    assert!(!loaded.policies.is_empty());

    // Procedure-level policy (on audit-log)
    let audit = loaded.procedures.iter().find(|p| {
        p.meta.id == make_id("procedure:audit-log")
    }).unwrap();
    assert!(!audit.policies.is_empty());

    // Step-level policy (on greet)
    let welcome = loaded.procedures.iter().find(|p| {
        p.meta.id == make_id("procedure:welcome")
    }).unwrap();
    assert!(!welcome.steps[0].policies.is_empty());
}

#[test]
fn round_trip_with_criteria_at_all_levels() {
    let dir = tempfile::tempdir().unwrap();
    let skill = build_full_skill();
    SkillWriter::write(dir.path(), &skill).unwrap();

    let loaded = SkillLoader::load(&dir.path().join("onboarding")).unwrap();

    // Skill-level criteria
    assert!(!loaded.criteria.is_empty());

    // Step-level criteria
    let welcome = loaded.procedures.iter().find(|p| {
        p.meta.id == make_id("procedure:welcome")
    }).unwrap();
    assert!(!welcome.steps[0].criteria.is_empty());
}
