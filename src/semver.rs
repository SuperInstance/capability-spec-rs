//! Semantic versioning with comparison, compatibility, and breaking-change detection.
//!
//! A minimal [`SemVer`] type supporting `major.minor.patch` parsing, ordering,
//! compatibility checks, and breaking-change detection. No pre-release or build
//! metadata — just the basics that fleet capability versioning needs.
//!
//! # Versioning Rules
//!
//! - **Compatible**: Same major version and minor ≥ target's minor.
//!   - `1.5.0` is compatible with `1.2.0` ✅
//!   - `2.0.0` is NOT compatible with `1.9.9` ❌
//!
//! - **Breaking**: Major version differs.
//!   - `2.0.0` is breaking relative to `1.9.9` ✅
//!   - `1.5.0` is NOT breaking relative to `1.2.0` ❌
//!
//! # Example
//!
//! ```rust
//! use capability_spec::semver::SemVer;
//!
//! let v = SemVer::parse("2.1.0").unwrap();
//! assert_eq!(v.to_string(), "2.1.0");
//!
//! assert!(SemVer::new(2, 0, 0) > SemVer::new(1, 9, 9));
//! assert!(SemVer::new(2, 0, 0).is_breaking(&SemVer::new(1, 0, 0)));
//! assert!(!SemVer::new(1, 5, 0).is_breaking(&SemVer::new(1, 2, 0)));
//! ```

use std::cmp::Ordering;
use std::fmt;

// ─────────────────────────────────────────────────────────────────────────────
// SemVer
// ─────────────────────────────────────────────────────────────────────────────

/// A semantic version (`major.minor.patch`).
///
/// Supports parsing, display, equality, ordering, compatibility checks, and
/// breaking-change detection. Implements the standard comparison traits so
/// versions can be sorted and compared naturally.
///
/// # Example
///
/// ```rust
/// use capability_spec::semver::SemVer;
///
/// let v = SemVer::new(1, 5, 2);
/// assert_eq!(v.major, 1);
/// assert_eq!(v.minor, 5);
/// assert_eq!(v.patch, 2);
/// assert_eq!(v.to_string(), "1.5.2");
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SemVer {
    /// Major version number. Changes here indicate breaking changes.
    pub major: u32,
    /// Minor version number. Changes here indicate backward-compatible additions.
    pub minor: u32,
    /// Patch version number. Changes here indicate backward-compatible fixes.
    pub patch: u32,
}

impl SemVer {
    /// Create a new semantic version from components.
    ///
    /// # Example
    ///
    /// ```rust
    /// use capability_spec::semver::SemVer;
    ///
    /// let v = SemVer::new(2, 1, 0);
    /// assert_eq!(v.to_string(), "2.1.0");
    /// ```
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Parse a semantic version from a string like `"1.2.3"` or `"v1.2.3"`.
    ///
    /// Returns `None` if the string doesn't match the expected format.
    ///
    /// # Example
    ///
    /// ```rust
    /// use capability_spec::semver::SemVer;
    ///
    /// assert_eq!(SemVer::parse("1.2.3"), Some(SemVer::new(1, 2, 3)));
    /// assert_eq!(SemVer::parse("v2.0.1"), Some(SemVer::new(2, 0, 1)));
    /// assert_eq!(SemVer::parse("invalid"), None);
    /// assert_eq!(SemVer::parse("1.2"), None);
    /// ```
    pub fn parse(s: &str) -> Option<Self> {
        // Strip optional 'v' prefix.
        let trimmed = s.trim_start_matches('v');
        let parts: Vec<&str> = trimmed.split('.').collect();
        if parts.len() != 3 {
            return None;
        }
        Some(Self {
            major: parts[0].parse().ok()?,
            minor: parts[1].parse().ok()?,
            patch: parts[2].parse().ok()?,
        })
    }

    /// Check if this version is **compatible** with `other`.
    ///
    /// Two versions are compatible if they share the same major version and
    /// this version's minor is greater than or equal to the other's.
    ///
    /// This follows the semver convention: same major = compatible API surface,
    /// higher minor = superset of features.
    ///
    /// # Example
    ///
    /// ```rust
    /// use capability_spec::semver::SemVer;
    ///
    /// // Same major, higher minor → compatible
    /// assert!(SemVer::new(1, 5, 0).is_compatible(&SemVer::new(1, 2, 0)));
    ///
    /// // Different major → not compatible
    /// assert!(!SemVer::new(2, 0, 0).is_compatible(&SemVer::new(1, 0, 0)));
    ///
    /// // Same version → compatible
    /// assert!(SemVer::new(1, 2, 0).is_compatible(&SemVer::new(1, 2, 0)));
    /// ```
    pub fn is_compatible(&self, other: &SemVer) -> bool {
        self.major == other.major && self.minor >= other.minor
    }

    /// Check if transitioning from this version to `other` would be a **breaking change**.
    ///
    /// A breaking change occurs when the major version differs. Per semver,
    /// a major version bump signals incompatible API changes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use capability_spec::semver::SemVer;
    ///
    /// // Major version change → breaking
    /// assert!(SemVer::new(2, 0, 0).is_breaking(&SemVer::new(1, 9, 9)));
    ///
    /// // Same major → not breaking
    /// assert!(!SemVer::new(1, 5, 0).is_breaking(&SemVer::new(1, 2, 0)));
    ///
    /// // Downgrade is also breaking if major differs
    /// assert!(SemVer::new(1, 0, 0).is_breaking(&SemVer::new(2, 0, 0)));
    /// ```
    pub fn is_breaking(&self, other: &SemVer) -> bool {
        self.major != other.major
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Trait implementations
// ─────────────────────────────────────────────────────────────────────────────

impl fmt::Display for SemVer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl PartialOrd for SemVer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemVer {
    /// Compare versions: major first, then minor, then patch.
    ///
    /// `2.0.0 > 1.9.9`, `1.3.0 > 1.2.9`, `1.2.1 > 1.2.0`.
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => self.patch.cmp(&other.patch),
                o => o,
            },
            o => o,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let v = SemVer::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_parse_v_prefix() {
        let v = SemVer::parse("v2.0.1").unwrap();
        assert_eq!(v.major, 2);
    }

    #[test]
    fn test_parse_invalid() {
        assert!(SemVer::parse("abc").is_none());
        assert!(SemVer::parse("1.2").is_none());
        assert!(SemVer::parse("").is_none());
        assert!(SemVer::parse("1.2.3.4").is_none());
    }

    #[test]
    fn test_display() {
        let v = SemVer::new(1, 2, 3);
        assert_eq!(format!("{}", v), "1.2.3");
    }

    #[test]
    fn test_ordering() {
        assert!(SemVer::new(2, 0, 0) > SemVer::new(1, 9, 9));
        assert!(SemVer::new(1, 2, 0) > SemVer::new(1, 1, 9));
        assert!(SemVer::new(1, 1, 2) > SemVer::new(1, 1, 1));
        assert!(SemVer::new(1, 1, 1) == SemVer::new(1, 1, 1));
    }

    #[test]
    fn test_compatible() {
        let v1 = SemVer::new(1, 2, 0);
        let v2 = SemVer::new(1, 3, 0);
        let v3 = SemVer::new(2, 0, 0);
        assert!(v2.is_compatible(&v1)); // 1.3 ≥ 1.2, same major
        assert!(!v3.is_compatible(&v1)); // different major
        assert!(v1.is_compatible(&v1)); // same version
    }

    #[test]
    fn test_is_breaking() {
        // Major version change → breaking
        assert!(SemVer::new(2, 0, 0).is_breaking(&SemVer::new(1, 9, 9)));
        assert!(SemVer::new(1, 0, 0).is_breaking(&SemVer::new(2, 0, 0)));

        // Same major → not breaking
        assert!(!SemVer::new(1, 5, 0).is_breaking(&SemVer::new(1, 2, 0)));
        assert!(!SemVer::new(1, 0, 0).is_breaking(&SemVer::new(1, 0, 0)));
    }
}
