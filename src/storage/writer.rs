//! SkillWriter — serialize a Skill to the directory layout (ADR-0006).

use std::path::Path;

use super::formats::{ProcedureFile, SkillFile, SkillManifest};
use super::StorageError;
use crate::types::Skill;

/// Writes a Skill to the ADR-0006 directory layout.
pub struct SkillWriter;

impl SkillWriter {
    /// Write a Skill to the directory structure at the given root path.
    ///
    /// Creates `<root>/<skill-slug>/skill.toml` and
    /// `<root>/<skill-slug>/procedures/<proc-slug>.toml` for each procedure.
    pub fn write(root: &Path, skill: &Skill) -> Result<(), StorageError> {
        let skill_slug = extract_slug(&skill.meta.id.to_string());
        let skill_dir = root.join(&skill_slug);
        let procedures_dir = skill_dir.join("procedures");

        std::fs::create_dir_all(&procedures_dir).map_err(|e| StorageError::IoError {
            path: procedures_dir.clone(),
            source: e,
        })?;

        // Remove stale procedure files before writing new ones.
        // Prevents a removed procedure from persisting on disk after re-write.
        if procedures_dir.exists() {
            for entry in std::fs::read_dir(&procedures_dir).map_err(|e| StorageError::IoError {
                path: procedures_dir.clone(),
                source: e,
            })? {
                let entry = entry.map_err(|e| StorageError::IoError {
                    path: procedures_dir.clone(),
                    source: e,
                })?;
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "toml") {
                    std::fs::remove_file(&path).map_err(|e| StorageError::IoError {
                        path: path.clone(),
                        source: e,
                    })?;
                }
            }
        }

        // Write skill.toml
        let manifest = SkillFile {
            skill: SkillManifest {
                meta: skill.meta.clone(),
                metadata: skill.metadata.clone(),
                policies: skill.policies.clone(),
                criteria: skill.criteria.clone(),
            },
        };
        let skill_toml_path = skill_dir.join("skill.toml");
        let content = toml::to_string_pretty(&manifest).map_err(|e| StorageError::SerializeError {
            path: skill_toml_path.clone(),
            source: e,
        })?;
        std::fs::write(&skill_toml_path, content).map_err(|e| StorageError::IoError {
            path: skill_toml_path,
            source: e,
        })?;

        // Write each procedure to procedures/<slug>.toml
        for proc in &skill.procedures {
            let proc_slug = extract_slug(&proc.meta.id.to_string());
            let proc_path = procedures_dir.join(format!("{proc_slug}.toml"));
            let proc_file = ProcedureFile {
                procedure: proc.clone(),
            };
            let content =
                toml::to_string_pretty(&proc_file).map_err(|e| StorageError::SerializeError {
                    path: proc_path.clone(),
                    source: e,
                })?;
            std::fs::write(&proc_path, content).map_err(|e| StorageError::IoError {
                path: proc_path,
                source: e,
            })?;
        }

        Ok(())
    }
}

/// Extract the slug from the last segment of an ItemId string.
/// Example: "skill:onboarding" → "onboarding", "procedure:auth-flow" → "auth-flow"
fn extract_slug(id_str: &str) -> String {
    let last_segment = id_str.rsplit('.').next().unwrap_or(id_str);
    last_segment
        .split_once(':')
        .map(|(_, slug)| slug.to_string())
        .unwrap_or_else(|| last_segment.to_string())
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

    #[test]
    fn extract_slug_from_single_segment() {
        assert_eq!(extract_slug("skill:onboarding"), "onboarding");
    }

    #[test]
    fn extract_slug_from_multi_segment() {
        assert_eq!(
            extract_slug("skill:test.procedure:auth-flow"),
            "auth-flow"
        );
    }

    #[test]
    fn write_creates_directory_structure() {
        let dir = tempfile::tempdir().unwrap();
        let skill = Skill {
            meta: make_meta("skill:onboarding"),
            metadata: SkillMeta {
                name: "Onboarding".to_string(),
                description: "Test".to_string(),
            },
            procedures: vec![Procedure {
                meta: make_meta("procedure:welcome"),
                steps: vec![],
                entrance_criteria: vec![],
                exit_criteria: vec![],
                policies: vec![],
                criteria: vec![],
            }],
            policies: vec![],
            criteria: vec![],
        };
        SkillWriter::write(dir.path(), &skill).unwrap();

        assert!(dir.path().join("onboarding/skill.toml").exists());
        assert!(dir.path().join("onboarding/procedures/welcome.toml").exists());
    }

    #[test]
    fn write_skill_toml_contains_no_procedures() {
        let dir = tempfile::tempdir().unwrap();
        let skill = Skill {
            meta: make_meta("skill:test"),
            metadata: SkillMeta::default(),
            procedures: vec![Procedure {
                meta: make_meta("procedure:p1"),
                steps: vec![],
                entrance_criteria: vec![],
                exit_criteria: vec![],
                policies: vec![],
                criteria: vec![],
            }],
            policies: vec![],
            criteria: vec![],
        };
        SkillWriter::write(dir.path(), &skill).unwrap();

        let content = std::fs::read_to_string(dir.path().join("test/skill.toml")).unwrap();
        assert!(!content.contains("[procedure"));
        assert!(!content.contains("procedures"));
    }

    #[test]
    fn write_zero_procedures_creates_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let skill = Skill {
            meta: make_meta("skill:empty"),
            metadata: SkillMeta::default(),
            procedures: vec![],
            policies: vec![],
            criteria: vec![],
        };
        SkillWriter::write(dir.path(), &skill).unwrap();

        assert!(dir.path().join("empty/procedures").is_dir());
        let entries: Vec<_> = std::fs::read_dir(dir.path().join("empty/procedures"))
            .unwrap()
            .collect();
        assert!(entries.is_empty());
    }

    #[test]
    fn written_toml_files_are_parseable() {
        let dir = tempfile::tempdir().unwrap();
        let skill = Skill {
            meta: make_meta("skill:test"),
            metadata: SkillMeta {
                name: "Test".to_string(),
                description: "A test skill".to_string(),
            },
            procedures: vec![],
            policies: vec![Policy {
                meta: make_meta("policy:no-secrets"),
                text: "Never expose secrets".to_string(),
                compatible_with: vec![],
            }],
            criteria: vec![],
        };
        SkillWriter::write(dir.path(), &skill).unwrap();

        let content = std::fs::read_to_string(dir.path().join("test/skill.toml")).unwrap();
        let _: SkillFile = toml::from_str(&content).unwrap();
    }

    use crate::types::{Policy, Procedure};
}
