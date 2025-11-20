//! Built-in expertises embedded in the binary
//!
//! This module contains admin-scope expertises that are pre-compiled into
//! the kanri-agent binary. Users can install them using `kanri-agent setup-builtin`.

/// Built-in expertise files (filename, content)
pub const BUILTIN_EXPERTISES: &[(&str, &str)] = &[(
    "manage-knowledge-with-kanri-agent.yaml",
    include_str!("../../../builtin/manage-knowledge-with-kanri-agent.yaml"),
)];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_expertises_not_empty() {
        assert!(
            !BUILTIN_EXPERTISES.is_empty(),
            "Should have at least one built-in expertise"
        );
    }

    #[test]
    fn test_builtin_expertise_content_valid() {
        for (filename, content) in BUILTIN_EXPERTISES {
            assert!(
                !content.is_empty(),
                "Expertise {} should not be empty",
                filename
            );
            assert!(
                content.contains("id:"),
                "Expertise {} should contain 'id:' field",
                filename
            );
            assert!(
                content.contains("scope: admin"),
                "Expertise {} should be admin scope",
                filename
            );
        }
    }
}
