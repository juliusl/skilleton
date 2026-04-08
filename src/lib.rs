//! Skilleton — a Rust-based tool for building and modifying agent skills.

/// Core item types for the skill hierarchy.
pub mod types;
/// Reference validation for cross-procedure invocations.
pub mod validate;
/// Policy conflict detection via scope-overlap reporting.
pub mod conflict;
/// Skill file I/O and repository pattern for storage/retrieval.
pub mod storage;
