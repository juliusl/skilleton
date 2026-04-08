//! Item identification primitives.
//!
//! Defines `ItemId` (hierarchical path-based identifiers), `TypePrefix`,
//! `Segment`, `CriterionRef`, and `SkillMeta`.

use std::fmt;

/// Valid type prefixes for item path segments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypePrefix {
    /// Root container for an agent skill.
    Skill,
    /// A list of Steps with entrance and exit Criteria.
    Procedure,
    /// A set of Tasks with completion Criteria.
    Step,
    /// A single instruction with a subject and action.
    Task,
    /// A constraint or rule that MUST be followed.
    Policy,
    /// A state or outcome that is either satisfied or unsatisfied.
    Criterion,
}

impl TypePrefix {
    fn as_str(&self) -> &'static str {
        match self {
            TypePrefix::Skill => "skill",
            TypePrefix::Procedure => "procedure",
            TypePrefix::Step => "step",
            TypePrefix::Task => "task",
            TypePrefix::Policy => "policy",
            TypePrefix::Criterion => "criterion",
        }
    }

    fn parse(s: &str) -> Result<Self, ItemIdError> {
        match s {
            "skill" => Ok(TypePrefix::Skill),
            "procedure" => Ok(TypePrefix::Procedure),
            "step" => Ok(TypePrefix::Step),
            "task" => Ok(TypePrefix::Task),
            "policy" => Ok(TypePrefix::Policy),
            "criterion" => Ok(TypePrefix::Criterion),
            _ => Err(ItemIdError::InvalidTypePrefix(s.to_string())),
        }
    }
}

impl fmt::Display for TypePrefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A parsed segment of an ItemId path.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Segment {
    /// The type classification of this segment.
    pub type_prefix: TypePrefix,
    /// The human-readable identifier within this segment.
    pub slug: String,
}

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.type_prefix, self.slug)
    }
}

/// Errors when parsing or constructing an ItemId.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemIdError {
    /// The input string was empty.
    Empty,
    /// The type prefix is not one of the valid prefixes.
    InvalidTypePrefix(String),
    /// The slug violates format constraints (charset, length, or hyphen rules).
    InvalidSlug(String),
    /// The segment could not be split into a `type:slug` pair.
    MalformedSegment(String),
}

impl fmt::Display for ItemIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ItemIdError::Empty => write!(f, "item ID cannot be empty"),
            ItemIdError::InvalidTypePrefix(p) => write!(f, "invalid type prefix: {p}"),
            ItemIdError::InvalidSlug(s) => write!(f, "invalid slug: {s}"),
            ItemIdError::MalformedSegment(s) => write!(f, "malformed segment: {s}"),
        }
    }
}

impl std::error::Error for ItemIdError {}

const MAX_SLUG_LEN: usize = 50;

fn validate_slug(slug: &str) -> Result<(), ItemIdError> {
    if slug.is_empty() || slug.len() > MAX_SLUG_LEN {
        return Err(ItemIdError::InvalidSlug(slug.to_string()));
    }
    if slug.starts_with('-') || slug.ends_with('-') || slug.contains("--") {
        return Err(ItemIdError::InvalidSlug(slug.to_string()));
    }
    for ch in slug.chars() {
        if !ch.is_ascii_lowercase() && ch != '-' && !ch.is_ascii_digit() {
            return Err(ItemIdError::InvalidSlug(slug.to_string()));
        }
    }
    Ok(())
}

/// Unique identifier for an item, structured as a hierarchical path.
/// Format: `type:slug` segments joined by `.` separators.
/// Example: `skill:my-skill.procedure:auth.step:validate`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemId {
    raw: String,
}

impl ItemId {
    /// Parse a string into a validated ItemId.
    pub fn parse(s: &str) -> Result<Self, ItemIdError> {
        if s.is_empty() {
            return Err(ItemIdError::Empty);
        }
        // Validate all segments
        for part in s.split('.') {
            let (prefix_str, slug) = part
                .split_once(':')
                .ok_or_else(|| ItemIdError::MalformedSegment(part.to_string()))?;
            TypePrefix::parse(prefix_str)?;
            validate_slug(slug)?;
        }
        Ok(ItemId { raw: s.to_string() })
    }

    /// Return the parsed segments of this path.
    pub fn segments(&self) -> Vec<Segment> {
        self.raw
            .split('.')
            .map(|part| {
                let (prefix_str, slug) = part.split_once(':').expect("validated at parse time");
                Segment {
                    type_prefix: TypePrefix::parse(prefix_str).expect("validated at parse time"),
                    slug: slug.to_string(),
                }
            })
            .collect()
    }

    /// Return the parent path (all segments except the last), or None if single-segment.
    pub fn parent(&self) -> Option<ItemId> {
        let last_dot = self.raw.rfind('.')?;
        Some(ItemId {
            raw: self.raw[..last_dot].to_string(),
        })
    }

    /// Check if this path starts with the given prefix path.
    pub fn prefix_matches(&self, prefix: &ItemId) -> bool {
        if self.raw == prefix.raw {
            return true;
        }
        self.raw.starts_with(&prefix.raw) && self.raw.as_bytes().get(prefix.raw.len()) == Some(&b'.')
    }

    /// Append a new segment to this path, returning a new ItemId.
    pub fn append(&self, type_prefix: TypePrefix, slug: &str) -> Result<ItemId, ItemIdError> {
        validate_slug(slug)?;
        let new_raw = format!("{}.{}:{}", self.raw, type_prefix, slug);
        Ok(ItemId { raw: new_raw })
    }

    /// Return the raw string representation.
    pub fn as_str(&self) -> &str {
        &self.raw
    }
}

impl fmt::Display for ItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.raw)
    }
}

/// Reference to a Criterion item by its ItemId.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriterionRef(pub ItemId);

/// Placeholder for agentskills.io specification metadata.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SkillMeta {
    pub name: String,
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_single_segment() {
        let id = ItemId::parse("skill:my-skill").unwrap();
        assert_eq!(id.to_string(), "skill:my-skill");
    }

    #[test]
    fn parse_valid_multi_segment() {
        let id = ItemId::parse("skill:my-skill.procedure:auth.step:validate").unwrap();
        assert_eq!(id.segments().len(), 3);
    }

    #[test]
    fn parse_rejects_invalid_prefix() {
        assert!(ItemId::parse("foo:bar").is_err());
    }

    #[test]
    fn parse_rejects_uppercase_slug() {
        assert!(ItemId::parse("skill:MySkill").is_err());
    }

    #[test]
    fn parse_rejects_slug_with_spaces() {
        assert!(ItemId::parse("skill:my skill").is_err());
    }

    #[test]
    fn parse_rejects_slug_over_50_chars() {
        let long_slug = "a".repeat(51);
        assert!(ItemId::parse(&format!("skill:{long_slug}")).is_err());
    }

    #[test]
    fn parse_rejects_empty_string() {
        assert!(ItemId::parse("").is_err());
    }

    #[test]
    fn parse_rejects_leading_hyphen_slug() {
        assert!(ItemId::parse("skill:-foo").is_err());
    }

    #[test]
    fn parse_rejects_trailing_hyphen_slug() {
        assert!(ItemId::parse("skill:foo-").is_err());
    }

    #[test]
    fn parse_rejects_consecutive_hyphens() {
        assert!(ItemId::parse("skill:foo--bar").is_err());
    }

    #[test]
    fn parse_rejects_bare_hyphen_slug() {
        assert!(ItemId::parse("skill:-").is_err());
    }

    #[test]
    fn parse_rejects_malformed_segment() {
        assert!(ItemId::parse("nocolon").is_err());
    }

    #[test]
    fn segments_returns_correct_parts() {
        let id = ItemId::parse("skill:s1.procedure:p1").unwrap();
        let segs = id.segments();
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].type_prefix, TypePrefix::Skill);
        assert_eq!(segs[0].slug, "s1");
        assert_eq!(segs[1].type_prefix, TypePrefix::Procedure);
        assert_eq!(segs[1].slug, "p1");
    }

    #[test]
    fn display_round_trips() {
        let original = "skill:s1.procedure:p1.step:s1";
        let id = ItemId::parse(original).unwrap();
        let reparsed = ItemId::parse(&id.to_string()).unwrap();
        assert_eq!(id, reparsed);
    }

    #[test]
    fn parent_returns_none_for_single_segment() {
        let id = ItemId::parse("skill:root").unwrap();
        assert!(id.parent().is_none());
    }

    #[test]
    fn parent_returns_parent_path() {
        let id = ItemId::parse("skill:s1.procedure:p1.step:s1").unwrap();
        let parent = id.parent().unwrap();
        assert_eq!(parent, ItemId::parse("skill:s1.procedure:p1").unwrap());
    }

    #[test]
    fn prefix_matches_self() {
        let id = ItemId::parse("skill:s1.procedure:p1").unwrap();
        assert!(id.prefix_matches(&id));
    }

    #[test]
    fn prefix_matches_ancestor() {
        let id = ItemId::parse("skill:s1.procedure:p1.step:s1").unwrap();
        let prefix = ItemId::parse("skill:s1").unwrap();
        assert!(id.prefix_matches(&prefix));
    }

    #[test]
    fn prefix_does_not_match_partial_segment() {
        let id = ItemId::parse("skill:s1-extra").unwrap();
        let prefix = ItemId::parse("skill:s1").unwrap();
        assert!(!id.prefix_matches(&prefix));
    }

    #[test]
    fn append_creates_child_path() {
        let parent = ItemId::parse("skill:s1").unwrap();
        let child = parent.append(TypePrefix::Procedure, "auth").unwrap();
        assert_eq!(child, ItemId::parse("skill:s1.procedure:auth").unwrap());
    }

    // -- QA plan findings --

    #[test]
    fn parse_accepts_exactly_50_char_slug() {
        let slug = "a".repeat(50);
        assert!(ItemId::parse(&format!("skill:{slug}")).is_ok());
    }

    #[test]
    fn parse_rejects_empty_slug_after_colon() {
        assert!(ItemId::parse("skill:").is_err());
    }

    #[test]
    fn parse_accepts_slug_with_digits() {
        assert!(ItemId::parse("skill:auth2").is_ok());
    }
}
