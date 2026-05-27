//! Semantic versioning.

use std::cmp::Ordering;
use std::fmt;

/// A semantic version (major.minor.patch).
#[derive(Debug, Clone, Default)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemVer {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.trim_start_matches('v').split('.').collect();
        if parts.len() != 3 {
            return None;
        }
        Some(Self {
            major: parts[0].parse().ok()?,
            minor: parts[1].parse().ok()?,
            patch: parts[2].parse().ok()?,
        })
    }

    pub fn is_compatible(&self, other: &SemVer) -> bool {
        self.major == other.major && self.minor >= other.minor
    }
}

impl fmt::Display for SemVer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl PartialEq for SemVer {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major && self.minor == other.minor && self.patch == other.patch
    }
}

impl Eq for SemVer {}

impl PartialOrd for SemVer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemVer {
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
    }

    #[test]
    fn test_compatible() {
        let v1 = SemVer::new(1, 2, 0);
        let v2 = SemVer::new(1, 3, 0);
        let v3 = SemVer::new(2, 0, 0);
        assert!(v2.is_compatible(&v1));
        assert!(!v3.is_compatible(&v1));
    }
}
