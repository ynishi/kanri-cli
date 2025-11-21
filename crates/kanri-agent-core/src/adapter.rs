//! Anti-corruption layer for llm-toolkit-expertise integration
//!
//! This module provides conversion traits between kanri-agent-core's Expertise
//! and llm-toolkit-expertise's types, ensuring that external dependencies don't
//! leak into our domain model.

use crate::expertise::{Expertise as KanriExpertise, KnowledgeFramework, OutputSchema};
use llm_toolkit_expertise::{
    Expertise as ToolkitExpertise, KnowledgeFragment, Priority, WeightedFragment,
};

/// Convert from kanri-agent-core's Expertise to llm-toolkit-expertise's Expertise
impl From<KanriExpertise> for ToolkitExpertise {
    fn from(kanri: KanriExpertise) -> Self {
        let mut expertise = ToolkitExpertise::new(kanri.id.clone(), kanri.version.clone());

        // Convert metadata tags
        expertise.tags.extend(kanri.metadata.tags.clone());

        // Add scope as a tag
        expertise.tags.push(format!(
            "scope:{}",
            kanri.metadata.scope.as_str()
        ));

        // Add groups as tags
        for group in &kanri.metadata.groups {
            expertise.tags.push(format!("group:{}", group));
        }

        // Convert the synthesis logic to a text fragment
        let synthesis_fragment = WeightedFragment::new(KnowledgeFragment::Text(
            format!(
                "# {}\n\n{}\n\n{}",
                kanri.name, kanri.description, kanri.body.synthesis_logic
            ),
        ))
        .with_priority(Priority::High);

        expertise.content.push(synthesis_fragment);

        // Convert input requirements if any
        for input_req in &kanri.body.input_requirements {
            let input_fragment = WeightedFragment::new(KnowledgeFragment::Text(format!(
                "## Input: {}\n\n{}\n\n**Format:** {}",
                input_req.name, input_req.description, input_req.format
            )))
            .with_priority(Priority::Normal);

            expertise.content.push(input_fragment);
        }

        // Convert output schema
        if !kanri.body.output_schema.sections.is_empty() {
            let mut output_text = format!(
                "## Output Schema\n\n{}\n\n",
                kanri.body.output_schema.description
            );

            for section in &kanri.body.output_schema.sections {
                output_text.push_str(&format!("### {}\n\n{}\n\n", section.title, section.description));

                if !section.required_points.is_empty() {
                    output_text.push_str("**Required Points:**\n");
                    for point in &section.required_points {
                        output_text.push_str(&format!("- {}\n", point));
                    }
                    output_text.push('\n');
                }
            }

            let output_fragment =
                WeightedFragment::new(KnowledgeFragment::Text(output_text))
                    .with_priority(Priority::Normal);

            expertise.content.push(output_fragment);
        }

        // Convert dependencies
        if !kanri.body.dependencies.is_empty() {
            let deps_text = format!(
                "## Dependencies\n\nThis expertise depends on:\n{}",
                kanri.body.dependencies
                    .iter()
                    .map(|d| format!("- {}", d))
                    .collect::<Vec<_>>()
                    .join("\n")
            );

            let deps_fragment = WeightedFragment::new(KnowledgeFragment::Text(deps_text))
                .with_priority(Priority::Low);

            expertise.content.push(deps_fragment);
        }

        expertise
    }
}

/// Convert from llm-toolkit-expertise's Expertise to kanri-agent-core's Expertise
impl From<ToolkitExpertise> for KanriExpertise {
    fn from(toolkit: ToolkitExpertise) -> Self {
        // Extract scope from tags (default to Personal if not found)
        let scope = toolkit
            .tags
            .iter()
            .find(|t| t.starts_with("scope:"))
            .and_then(|t| t.strip_prefix("scope:"))
            .and_then(|s| crate::expertise::Scope::from_str(s).ok())
            .unwrap_or(crate::expertise::Scope::Personal);

        // Extract groups from tags
        let groups: Vec<String> = toolkit
            .tags
            .iter()
            .filter(|t| t.starts_with("group:"))
            .filter_map(|t| t.strip_prefix("group:").map(|s| s.to_string()))
            .collect();

        // Extract other tags (excluding scope and group)
        let tags: Vec<String> = toolkit
            .tags
            .iter()
            .filter(|t| !t.starts_with("scope:") && !t.starts_with("group:"))
            .cloned()
            .collect();

        // Combine all fragments into synthesis_logic
        let synthesis_logic = toolkit
            .content
            .iter()
            .map(|wf| wf.fragment.to_prompt())
            .collect::<Vec<_>>()
            .join("\n---\n\n");

        let mut expertise = KanriExpertise::new(toolkit.id.clone(), toolkit.id.clone(), scope);
        expertise.version = toolkit.version;
        expertise.metadata.groups = groups;
        expertise.metadata.tags = tags;
        expertise.body = KnowledgeFramework {
            input_requirements: Vec::new(),
            output_schema: OutputSchema {
                description: "Converted from llm-toolkit-expertise".to_string(),
                sections: Vec::new(),
            },
            dependencies: Vec::new(),
            synthesis_logic,
        };

        expertise
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expertise::{
        Activation, ActivationMode, ExpertiseMetadata, InputRequirement, Scope,
        SectionDefinition, Visibility,
    };
    use chrono::Utc;

    #[test]
    fn test_kanri_to_toolkit_conversion() {
        let kanri = KanriExpertise {
            id: "test-expertise".to_string(),
            version: "1.0.0".to_string(),
            name: "Test Expertise".to_string(),
            description: "A test expertise".to_string(),
            metadata: ExpertiseMetadata {
                scope: Scope::Company,
                visibility: Visibility::Private,
                groups: vec!["backend".to_string(), "backend/api".to_string()],
                author: "test".to_string(),
                created: Utc::now(),
                updated: Utc::now(),
                tags: vec!["testing".to_string()],
            },
            activation: Activation {
                mode: ActivationMode::Skill,
                triggers: vec![],
                auto: false,
                contexts: vec![Scope::Company],
            },
            inheritance: None,
            parameters: std::collections::HashMap::new(),
            body: KnowledgeFramework {
                input_requirements: vec![InputRequirement {
                    name: "user_query".to_string(),
                    description: "The user's query".to_string(),
                    format: "text".to_string(),
                }],
                output_schema: OutputSchema {
                    description: "Test output".to_string(),
                    sections: vec![SectionDefinition {
                        title: "Result".to_string(),
                        description: "The result".to_string(),
                        required_points: vec!["Point 1".to_string()],
                    }],
                },
                dependencies: vec!["other-expertise".to_string()],
                synthesis_logic: "# Logic\n\nDo something".to_string(),
            },
        };

        let toolkit: ToolkitExpertise = kanri.into();

        assert_eq!(toolkit.id, "test-expertise");
        assert_eq!(toolkit.version, "1.0.0");
        assert!(toolkit.tags.contains(&"testing".to_string()));
        assert!(toolkit.tags.contains(&"scope:company".to_string()));
        assert!(toolkit.tags.contains(&"group:backend".to_string()));
        assert!(!toolkit.content.is_empty());
    }

    #[test]
    fn test_toolkit_to_kanri_conversion() {
        let toolkit = ToolkitExpertise::new("test-expertise", "1.0.0")
            .with_tag("scope:company")
            .with_tag("group:backend")
            .with_tag("testing")
            .with_fragment(
                WeightedFragment::new(KnowledgeFragment::Text("Test content".to_string()))
                    .with_priority(Priority::High),
            );

        let kanri: KanriExpertise = toolkit.into();

        assert_eq!(kanri.id, "test-expertise");
        assert_eq!(kanri.version, "1.0.0");
        assert_eq!(kanri.metadata.scope, Scope::Company);
        assert!(kanri.metadata.groups.contains(&"backend".to_string()));
        assert!(kanri.metadata.tags.contains(&"testing".to_string()));
        assert!(kanri.body.synthesis_logic.contains("Test content"));
    }

    #[test]
    fn test_roundtrip_preserves_core_data() {
        let original = KanriExpertise::new(
            "test".to_string(),
            "Test Expertise".to_string(),
            Scope::Personal,
        );

        let toolkit: ToolkitExpertise = original.clone().into();
        let restored: KanriExpertise = toolkit.into();

        assert_eq!(restored.id, original.id);
        assert_eq!(restored.metadata.scope, original.metadata.scope);
    }
}
