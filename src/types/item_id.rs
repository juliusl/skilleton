use std::fmt;

/// Valid type prefixes for item path segments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypePrefix {
    Skill,
    Procedure,
    Step,
    Task,
    Policy,
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
    pub type_prefix: TypePrefix,
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
    Empty,
    InvalidTypePrefix(String),
    InvalidSlug(String),
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

