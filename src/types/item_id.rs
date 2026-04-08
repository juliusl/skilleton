//! Item identification primitives.
//!
//! Defines `ItemId` (hierarchical path-based identifiers), `TypePrefix`,
//! `Segment`, `CriterionRef`, and `SkillMeta`.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
    pub fn as_str(&self) -> &'static str {
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

    /// Return the type prefix of the last segment.
    ///
    /// For single-segment IDs like `criterion:ready`, returns `Criterion`.
    /// For multi-segment IDs like `skill:s.procedure:auth`, returns `Procedure`.
    pub fn type_prefix(&self) -> TypePrefix {
        let last_segment = self.raw.rsplit('.').next().unwrap_or(&self.raw);
        let (prefix_str, _) = last_segment.split_once(':').expect("validated at parse time");
        TypePrefix::parse(prefix_str).expect("validated at parse time")
    }
}

impl fmt::Display for ItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.raw)
    }
}

impl Serialize for ItemId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ItemId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        ItemId::parse(&s).map_err(serde::de::Error::custom)
    }
}

/// Type-safe reference to a Criterion item by its [`ItemId`].
///
/// The inner `ItemId` must have [`TypePrefix::Criterion`]. Use [`CriterionRef::new`]
/// to construct with validation, or [`CriterionRef::new_unchecked`] when the prefix
/// is already known (e.g., in test helpers).
///
/// **Referential integrity is the caller's responsibility** — this type guarantees
/// the correct type-prefix but does not verify that the referenced Criterion
/// actually exists in any Skill or scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriterionRef(ItemId);

impl CriterionRef {
    /// Create a new `CriterionRef`, validating that the `ItemId` has a `criterion:` prefix.
    pub fn new(id: ItemId) -> Result<Self, ItemIdError> {
        let first_segment = id.segments().first().map(|s| s.type_prefix);
        if first_segment != Some(TypePrefix::Criterion) {
            return Err(ItemIdError::InvalidTypePrefix(format!(
                "expected criterion prefix, got {}",
                id.as_str()
            )));
        }
        Ok(CriterionRef(id))
    }

    /// Create a `CriterionRef` without validating the type prefix.
    /// Use only when the prefix is already known to be `criterion:`.
    pub fn new_unchecked(id: ItemId) -> Self {
        CriterionRef(id)
    }

    /// Return a reference to the inner `ItemId`.
    pub fn id(&self) -> &ItemId {
        &self.0
    }
}

impl Serialize for CriterionRef {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.id().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CriterionRef {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let id = ItemId::deserialize(deserializer)?;
        CriterionRef::new(id).map_err(serde::de::Error::custom)
    }
}

/// Placeholder for agentskills.io specification metadata.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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

    #[test]
    fn serde_item_id_round_trips_as_string() {
        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        struct Wrapper { id: ItemId }
        let w = Wrapper { id: ItemId::parse("skill:onboarding.procedure:auth").unwrap() };
        let serialized = toml::to_string(&w).unwrap();
        assert!(serialized.contains("skill:onboarding.procedure:auth"));
        let deserialized: Wrapper = toml::from_str(&serialized).unwrap();
        assert_eq!(w, deserialized);
    }

    #[test]
    fn serde_item_id_serializes_as_string_not_struct() {
        #[derive(serde::Serialize)]
        struct Wrapper { id: ItemId }
        let w = Wrapper { id: ItemId::parse("skill:onboarding").unwrap() };
        let serialized = toml::to_string(&w).unwrap();
        assert!(!serialized.contains("[id]"));
        assert!(serialized.starts_with("id = \"skill:onboarding\""));
    }

    #[test]
    fn serde_item_id_rejects_malformed_string() {
        #[derive(serde::Deserialize)]
        struct Wrapper { id: ItemId }
        let result: Result<Wrapper, _> = toml::from_str("id = \"invalid\"");
        assert!(result.is_err());
    }

    #[test]
    fn serde_criterion_ref_round_trips() {
        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        struct Wrapper { cref: CriterionRef }
        let w = Wrapper { cref: CriterionRef::new(ItemId::parse("criterion:enabled").unwrap()).unwrap() };
        let serialized = toml::to_string(&w).unwrap();
        let deserialized: Wrapper = toml::from_str(&serialized).unwrap();
        assert_eq!(w, deserialized);
    }

    #[test]
    fn serde_skill_meta_round_trips() {
        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        struct Wrapper { meta: SkillMeta }
        let w = Wrapper {
            meta: SkillMeta {
                name: "Onboarding".to_string(),
                description: "New user onboarding flow".to_string(),
            },
        };
        let serialized = toml::to_string(&w).unwrap();
        let deserialized: Wrapper = toml::from_str(&serialized).unwrap();
        assert_eq!(w, deserialized);
    }

    // -- Milestone 1 & 2 follow-up tests --

    #[test]
    fn type_prefix_as_str_is_public() {
        assert_eq!(TypePrefix::Skill.as_str(), "skill");
        assert_eq!(TypePrefix::Procedure.as_str(), "procedure");
        assert_eq!(TypePrefix::Step.as_str(), "step");
        assert_eq!(TypePrefix::Task.as_str(), "task");
        assert_eq!(TypePrefix::Policy.as_str(), "policy");
        assert_eq!(TypePrefix::Criterion.as_str(), "criterion");
    }

    #[test]
    fn criterion_ref_new_accepts_criterion_prefix() {
        let id = ItemId::parse("criterion:enabled").unwrap();
        let cref = CriterionRef::new(id.clone()).unwrap();
        assert_eq!(cref.id(), &id);
    }

    #[test]
    fn criterion_ref_new_rejects_non_criterion_prefix() {
        let id = ItemId::parse("policy:no-secrets").unwrap();
        assert!(CriterionRef::new(id).is_err());
    }

    #[test]
    fn criterion_ref_new_rejects_procedure_prefix() {
        let id = ItemId::parse("procedure:auth").unwrap();
        assert!(CriterionRef::new(id).is_err());
    }

    #[test]
    fn criterion_ref_new_unchecked_bypasses_validation() {
        let id = ItemId::parse("policy:no-secrets").unwrap();
        let cref = CriterionRef::new_unchecked(id.clone());
        assert_eq!(cref.id(), &id);
    }

    #[test]
    fn criterion_ref_id_accessor_returns_inner() {
        let id = ItemId::parse("criterion:ready").unwrap();
        let cref = CriterionRef::new(id.clone()).unwrap();
        assert_eq!(cref.id(), &id);
        assert_eq!(cref.id().as_str(), "criterion:ready");
    }

    #[test]
    fn serde_criterion_ref_rejects_non_criterion_prefix() {
        #[derive(serde::Deserialize)]
        struct Wrapper { _cref: CriterionRef }
        let result: Result<Wrapper, _> = toml::from_str(r#"_cref = "policy:no-secrets""#);
        assert!(result.is_err());
    }
}
