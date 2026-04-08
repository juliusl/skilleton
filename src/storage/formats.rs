//! Shared serialization types for the skill file layout (ADR-0006).

use serde::{Deserialize, Serialize};

use crate::types::{Criterion, ItemMeta, Policy, Procedure, SkillMeta};

/// Wrapper for skill.toml — skill metadata, policies, and criteria; no procedures.
#[derive(Serialize, Deserialize)]
pub(crate) struct SkillFile {
    pub(crate) skill: SkillManifest,
}

/// Skill metadata for skill.toml (Skill minus procedures).
#[derive(Serialize, Deserialize)]
pub(crate) struct SkillManifest {
    #[serde(flatten)]
    pub(crate) meta: ItemMeta,
    #[serde(flatten)]
    pub(crate) metadata: SkillMeta,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) policies: Vec<Policy>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) criteria: Vec<Criterion>,
}

/// Wrapper for procedures/<slug>.toml.
#[derive(Serialize, Deserialize)]
pub(crate) struct ProcedureFile {
    pub(crate) procedure: Procedure,
}
