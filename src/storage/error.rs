//! Storage error types.

use std::fmt;
use std::path::PathBuf;

/// Errors from skill storage operations.
#[derive(Debug)]
pub enum StorageError {
    /// skill.toml not found at the expected path.
    MissingSkillFile(PathBuf),
    /// TOML parse error with file path context.
    ParseError {
        path: PathBuf,
        source: toml::de::Error,
    },
    /// TOML serialization error with file path context.
    SerializeError {
        path: PathBuf,
        source: toml::ser::Error,
    },
    /// I/O error with file path context.
    IoError {
        path: PathBuf,
        source: std::io::Error,
    },
    /// A procedure file's name does not match the slug from its ItemId.
    SlugFilenameMismatch {
        path: PathBuf,
        filename_stem: String,
        item_slug: String,
    },
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::MissingSkillFile(path) => {
                write!(f, "skill.toml not found at {}", path.display())
            }
            StorageError::ParseError { path, source } => {
                write!(f, "failed to parse {}: {source}", path.display())
            }
            StorageError::SerializeError { path, source } => {
                write!(f, "failed to serialize {}: {source}", path.display())
            }
            StorageError::IoError { path, source } => {
                write!(f, "I/O error at {}: {source}", path.display())
            }
            StorageError::SlugFilenameMismatch {
                path,
                filename_stem,
                item_slug,
            } => {
                write!(
                    f,
                    "slug-filename mismatch in {}: filename stem '{}' does not match ItemId slug '{}'",
                    path.display(),
                    filename_stem,
                    item_slug
                )
            }
        }
    }
}

impl std::error::Error for StorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StorageError::ParseError { source, .. } => Some(source),
            StorageError::SerializeError { source, .. } => Some(source),
            StorageError::IoError { source, .. } => Some(source),
            StorageError::MissingSkillFile(_) => None,
            StorageError::SlugFilenameMismatch { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_skill_file_display_includes_path() {
        let err = StorageError::MissingSkillFile(PathBuf::from("/tmp/test/skill.toml"));
        let msg = format!("{err}");
        assert!(msg.contains("/tmp/test/skill.toml"));
        assert!(msg.contains("not found"));
    }

    #[test]
    fn storage_error_implements_error_trait() {
        let err = StorageError::MissingSkillFile(PathBuf::from("test"));
        let _: &dyn std::error::Error = &err;
    }
}
