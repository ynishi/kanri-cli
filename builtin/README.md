# Admin Expertises

This directory contains official, public-facing expertise documents for the kanri-agent project.

## Purpose

The `admin` scope is used for:
- Official documentation and guides
- Public-facing expertise suitable for general users
- Reference materials that can be distributed with kanri-agent

## Contents

### manage-knowledge-with-kanri-agent.yaml

Practical operational guide for maintaining high-quality expertise and skill management with kanri-agent.

**Key Topics:**
- Schema verification (critical for expertise creation)
- Expertise creation checklist (7-phase workflow)
- Common pitfalls (Top 5 mistakes to avoid)
- Quality standards (minimum and recommended criteria)
- Update and versioning workflows
- Troubleshooting guide
- Quick reference commands

## Characteristics

All expertises in this directory:
- ✅ Are free from internal/proprietary references
- ✅ Use generic examples applicable to any project
- ✅ Follow best practices for documentation
- ✅ Are suitable for public distribution
- ✅ Use `scope: admin` and `visibility: public`

## Usage

These expertises can be:
1. Deployed to Claude Code globally or locally
2. Shared with other kanri-agent users
3. Referenced in documentation
4. Used as templates for creating new expertises

## Maintenance

When adding new admin expertises:
1. Ensure all content is generic and public-facing
2. Remove any company-specific or internal references
3. Use clear, descriptive names and tags
4. Follow the established structure and format
5. Test deployment before committing

## Future: Built-in Expertise Distribution

**Current limitation:** Admin expertises require git clone to access.

**Planned feature:**
```bash
# Embed admin expertises in kanri-agent binary
kanri-agent setup-builtin [--target global|local]

# Automatically deploys built-in admin expertises
# → ~/.kanri-agent/cache/admin/
# → Ready to use immediately after installation
```

**Implementation approach:**
- Use `include_str!()` or `rust-embed` crate to embed YAML files
- Add `setup-builtin` subcommand to CLI
- Design command structure considering consistency with other commands (init, deploy, clean, etc.)
