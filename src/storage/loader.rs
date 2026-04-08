//! SkillLoader — read and assemble a Skill from the directory layout (ADR-0006).

use std::path::Path;

use super::formats::{ProcedureFile, SkillFile};
use super::StorageError;
use crate::types::Skill;

/// Reads a Skill from the ADR-0006 directory layout.
pub struct SkillLoader;

impl SkillLoader {
    /// Load a Skill from the directory at the given path.
    ///
    /// `skill_dir` should point to the skill directory (e.g., `skills/onboarding/`).
    /// Reads `skill.toml` then iterates `procedures/*.toml` and assembles the full Skill.
    pub fn load(skill_dir: &Path) -> Result<Skill, StorageError> {
        let skill_toml_path = skill_dir.join("skill.toml");

        if !skill_toml_path.exists() {
            return Err(StorageError::MissingSkillFile(skill_toml_path));
        }

        let content = std::fs::read_to_string(&skill_toml_path).map_err(|e| {
            StorageError::IoError {
                path: skill_toml_path.clone(),
                source: e,
            }
        })?;

        let skill_file: SkillFile =
            toml::from_str(&content).map_err(|e| StorageError::ParseError {
                path: skill_toml_path,
                source: e,
            })?;

        // Load procedures from procedures/*.toml
        let procedures_dir = skill_dir.join("procedures");
        let mut procedures = Vec::new();

        if procedures_dir.is_dir() {
            let read_entries = std::fs::read_dir(&procedures_dir)
                .map_err(|e| StorageError::IoError {
                    path: procedures_dir.clone(),
                    source: e,
                })?;

            let mut entries = Vec::new();
            for entry in read_entries {
                let entry = entry.map_err(|e| StorageError::IoError {
                    path: procedures_dir.clone(),
                    source: e,
                })?;
                if entry.path().extension().is_some_and(|ext| ext == "toml") {
                    entries.push(entry);
                }
            }

            // Sort by filename for deterministic ordering across platforms.
            entries.sort_by_key(|e| e.file_name());

            for entry in entries {
                let proc_path = entry.path();
                let proc_content =
                    std::fs::read_to_string(&proc_path).map_err(|e| StorageError::IoError {
                        path: proc_path.clone(),
                        source: e,
                    })?;
                let proc_file: ProcedureFile =
                    toml::from_str(&proc_content).map_err(|e| StorageError::ParseError {
                        path: proc_path,
                        source: e,
                    })?;
                procedures.push(proc_file.procedure);
            }
        }

        Ok(Skill {
            meta: skill_file.skill.meta,
            metadata: skill_file.skill.metadata,
            procedures,
            policies: skill_file.skill.policies,
            criteria: skill_file.skill.criteria,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ItemId, ItemMeta, SkillMeta, Policy, Procedure};
    use crate::storage::SkillWriter;

    fn make_id(s: &str) -> ItemId {
        ItemId::parse(s).expect("test ID should be valid")
    }

    fn make_meta(s: &str) -> ItemMeta {
        ItemMeta { id: make_id(s), conditions: vec![] }
    }

    fn make_skill_with_procedures(procs: Vec<Procedure>) -> Skill {
        Skill {
            meta: make_meta("skill:test"),
            metadata: SkillMeta {
                name: "Test".to_string(),
                description: "A test skill".to_string(),
            },
            procedures: procs,
            policies: vec![],
            criteria: vec![],
        }
    }

    #[test]
    fn load_assembles_multi_file_skill() {
        let dir = tempfile::tempdir().unwrap();
        let proc = Procedure {
            meta: make_meta("procedure:welcome"),
            steps: vec![],
            entrance_criteria: vec![],
            exit_criteria: vec![],
            policies: vec![],
            criteria: vec![],
        };
        let skill = make_skill_with_procedures(vec![proc]);
        SkillWriter::write(dir.path(), &skill).unwrap();

        let loaded = SkillLoader::load(&dir.path().join("test")).unwrap();
        assert_eq!(loaded.meta.id, make_id("skill:test"));
        assert_eq!(loaded.procedures.len(), 1);
        assert_eq!(loaded.procedures[0].meta.id, make_id("procedure:welcome"));
    }

    #[test]
    fn load_missing_skill_toml_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let result = SkillLoader::load(&dir.path().join("nonexistent"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, StorageError::MissingSkillFile(_)));
    }

    #[test]
    fn load_malformed_toml_returns_parse_error() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path().join("bad");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("skill.toml"), "not valid toml {{{").unwrap();

        let result = SkillLoader::load(&skill_dir);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, StorageError::ParseError { .. }));
        let msg = format!("{err}");
        assert!(msg.contains("skill.toml"));
    }

    #[test]
    fn load_empty_procedures_produces_zero_procedures() {
        let dir = tempfile::tempdir().unwrap();
        let skill = make_skill_with_procedures(vec![]);
        SkillWriter::write(dir.path(), &skill).unwrap();

        let loaded = SkillLoader::load(&dir.path().join("test")).unwrap();
        assert!(loaded.procedures.is_empty());
    }

    #[test]
    fn load_slug_filename_consistent_with_writer() {
        let dir = tempfile::tempdir().unwrap();
        let proc = Procedure {
            meta: make_meta("procedure:auth-flow"),
            steps: vec![],
            entrance_criteria: vec![],
            exit_criteria: vec![],
            policies: vec![],
            criteria: vec![],
        };
        let skill = make_skill_with_procedures(vec![proc]);
        SkillWriter::write(dir.path(), &skill).unwrap();

        // Verify the file exists with the expected name
        assert!(dir.path().join("test/procedures/auth-flow.toml").exists());

        let loaded = SkillLoader::load(&dir.path().join("test")).unwrap();
        assert_eq!(loaded.procedures[0].meta.id, make_id("procedure:auth-flow"));
    }

    #[test]
    fn load_ignores_non_toml_files() {
        let dir = tempfile::tempdir().unwrap();
        let skill = make_skill_with_procedures(vec![]);
        SkillWriter::write(dir.path(), &skill).unwrap();

        // Add a non-TOML file to procedures/
        std::fs::write(dir.path().join("test/procedures/.gitkeep"), "").unwrap();
        std::fs::write(dir.path().join("test/procedures/README.md"), "# Info").unwrap();

        let loaded = SkillLoader::load(&dir.path().join("test")).unwrap();
        assert!(loaded.procedures.is_empty());
    }

    #[test]
    fn load_produces_deterministic_order() {
        let dir = tempfile::tempdir().unwrap();
        let skill = Skill {
            meta: make_meta("skill:test"),
            metadata: SkillMeta::default(),
            procedures: vec![
                Procedure {
                    meta: make_meta("procedure:zebra"),
                    steps: vec![],
                    entrance_criteria: vec![],
                    exit_criteria: vec![],
                    policies: vec![],
                    criteria: vec![],
                },
                Procedure {
                    meta: make_meta("procedure:alpha"),
                    steps: vec![],
                    entrance_criteria: vec![],
                    exit_criteria: vec![],
                    policies: vec![],
                    criteria: vec![],
                },
            ],
            policies: vec![],
            criteria: vec![],
        };
        SkillWriter::write(dir.path(), &skill).unwrap();

        // Load twice and verify same order (alphabetical by filename)
        let loaded1 = SkillLoader::load(&dir.path().join("test")).unwrap();
        let loaded2 = SkillLoader::load(&dir.path().join("test")).unwrap();
        assert_eq!(loaded1.procedures[0].meta.id, loaded2.procedures[0].meta.id);
        // Should be sorted alphabetically: alpha before zebra
        assert_eq!(loaded1.procedures[0].meta.id, make_id("procedure:alpha"));
        assert_eq!(loaded1.procedures[1].meta.id, make_id("procedure:zebra"));
    }
}
