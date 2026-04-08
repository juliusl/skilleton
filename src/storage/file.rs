//! FileRepository — file-system-backed skill storage (ADR-0008).

use std::path::PathBuf;

use super::loader::SkillLoader;
use super::traits::{find_item_in_skill, ItemRef, RepositoryError, SkillRepository};
use super::StorageError;
use crate::types::{ItemId, Skill};

/// File-system-backed repository. Uses SkillLoader for individual skill loading.
pub struct FileRepository {
    root: PathBuf,
}

impl FileRepository {
    /// Create a new FileRepository rooted at the given directory.
    pub fn new(root: PathBuf) -> Self {
        FileRepository { root }
    }
}

impl SkillRepository for FileRepository {
    fn load_skill(&self, name: &str) -> Result<Skill, RepositoryError> {
        // Reject names with path separators or traversal sequences.
        if name.contains('/') || name.contains('\\') || name.contains("..") || name.is_empty() {
            return Err(RepositoryError::NotFound(name.to_string()));
        }
        let skill_dir = self.root.join(name);
        SkillLoader::load(&skill_dir).map_err(RepositoryError::Storage)
    }

    fn list_skills(&self) -> Result<Vec<String>, RepositoryError> {
        if !self.root.is_dir() {
            return Ok(Vec::new());
        }

        let mut names = Vec::new();
        let entries = std::fs::read_dir(&self.root).map_err(|e| {
            RepositoryError::Storage(StorageError::IoError {
                path: self.root.clone(),
                source: e,
            })
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                RepositoryError::Storage(StorageError::IoError {
                    path: self.root.clone(),
                    source: e,
                })
            })?;
            let path = entry.path();
            if path.is_dir() && path.join("skill.toml").exists() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    names.push(name.to_string());
                }
            }
        }

        names.sort();
        Ok(names)
    }

    fn find_item(&self, skill: &str, id: &ItemId) -> Result<Option<ItemRef>, RepositoryError> {
        let skill_data = self.load_skill(skill)?;
        Ok(find_item_in_skill(&skill_data, id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use crate::storage::SkillWriter;

    fn make_id(s: &str) -> ItemId {
        ItemId::parse(s).expect("test ID should be valid")
    }

    fn make_meta(s: &str) -> ItemMeta {
        ItemMeta { id: make_id(s), conditions: vec![] }
    }

    fn make_test_skill(name: &str) -> Skill {
        Skill {
            meta: make_meta(&format!("skill:{name}")),
            metadata: SkillMeta {
                name: name.to_string(),
                description: "test".to_string(),
            },
            procedures: vec![],
            policies: vec![],
            criteria: vec![],
        }
    }

    #[test]
    fn load_skill_from_disk() {
        let dir = tempfile::tempdir().unwrap();
        SkillWriter::write(dir.path(), &make_test_skill("alpha")).unwrap();

        let repo = FileRepository::new(dir.path().to_path_buf());
        let skill = repo.load_skill("alpha").unwrap();
        assert_eq!(skill.meta.id, make_id("skill:alpha"));
    }

    #[test]
    fn list_skills_returns_all() {
        let dir = tempfile::tempdir().unwrap();
        SkillWriter::write(dir.path(), &make_test_skill("alpha")).unwrap();
        SkillWriter::write(dir.path(), &make_test_skill("beta")).unwrap();

        let repo = FileRepository::new(dir.path().to_path_buf());
        let names = repo.list_skills().unwrap();
        assert_eq!(names, vec!["alpha", "beta"]);
    }

    #[test]
    fn list_skills_empty_root() {
        let dir = tempfile::tempdir().unwrap();
        let repo = FileRepository::new(dir.path().to_path_buf());
        let names = repo.list_skills().unwrap();
        assert!(names.is_empty());
    }

    #[test]
    fn list_skills_nonexistent_root() {
        let repo = FileRepository::new(PathBuf::from("/tmp/nonexistent-skilleton-test-dir"));
        let names = repo.list_skills().unwrap();
        assert!(names.is_empty());
    }

    #[test]
    fn find_item_by_id() {
        let dir = tempfile::tempdir().unwrap();
        SkillWriter::write(dir.path(), &make_test_skill("test")).unwrap();

        let repo = FileRepository::new(dir.path().to_path_buf());
        let item = repo.find_item("test", &make_id("skill:test")).unwrap();
        assert!(matches!(item, Some(ItemRef::Skill(_))));
    }

    #[test]
    fn load_malformed_skill_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let bad_dir = dir.path().join("bad");
        std::fs::create_dir_all(&bad_dir).unwrap();
        std::fs::write(bad_dir.join("skill.toml"), "not valid").unwrap();

        let repo = FileRepository::new(dir.path().to_path_buf());
        let result = repo.load_skill("bad");
        assert!(result.is_err());
    }

    #[test]
    fn load_skill_rejects_path_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let repo = FileRepository::new(dir.path().to_path_buf());
        assert!(repo.load_skill("../../etc").is_err());
        assert!(repo.load_skill("../secret").is_err());
        assert!(repo.load_skill("foo/bar").is_err());
        assert!(repo.load_skill("").is_err());
    }
}
