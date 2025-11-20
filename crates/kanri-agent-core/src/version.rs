use serde::{Deserialize, Serialize};
use std::fmt;

/// セマンティックバージョン
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn parse(s: &str) -> crate::Result<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(crate::Error::Config(format!("Invalid version: {}", s)));
        }

        let major = parts[0]
            .parse()
            .map_err(|_| crate::Error::Config(format!("Invalid major version: {}", parts[0])))?;
        let minor = parts[1]
            .parse()
            .map_err(|_| crate::Error::Config(format!("Invalid minor version: {}", parts[1])))?;
        let patch = parts[2]
            .parse()
            .map_err(|_| crate::Error::Config(format!("Invalid patch version: {}", parts[2])))?;

        Ok(Version::new(major, minor, patch))
    }

    pub fn bump_major(&self) -> Self {
        Self::new(self.major + 1, 0, 0)
    }

    pub fn bump_minor(&self) -> Self {
        Self::new(self.major, self.minor + 1, 0)
    }

    pub fn bump_patch(&self) -> Self {
        Self::new(self.major, self.minor, self.patch + 1)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Default for Version {
    fn default() -> Self {
        Self::new(0, 1, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parse() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_version_display() {
        let v = Version::new(2, 5, 10);
        assert_eq!(v.to_string(), "2.5.10");
    }

    #[test]
    fn test_version_bump() {
        let v = Version::new(1, 2, 3);

        assert_eq!(v.bump_major().to_string(), "2.0.0");
        assert_eq!(v.bump_minor().to_string(), "1.3.0");
        assert_eq!(v.bump_patch().to_string(), "1.2.4");
    }

    #[test]
    fn test_version_comparison() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(1, 2, 0);
        let v3 = Version::new(2, 0, 0);

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3);
    }
}
