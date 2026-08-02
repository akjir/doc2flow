<!--
===============================================================================
DOC2FLOW (D2F) {{APP_VERSION}} - TEMPLATE & USAGE GUIDE
Repository: https://github.com/akjir/doc2flow
License: GPL-3.0-or-later
===============================================================================

1. FRONTMATTER METADATA:
   - title: Main document title.
   - subtitle: Subtitle or document description.
   - date: Protocol date (YYYY-MM-DD).
   - version: Document version.
   - language / lang: Language code for static UI translation ('en', 'de').
   - logo: Optional path to a custom logo image (SVG, PNG, JPG, WebP).
   - numbered_sections: Enable automatic section numbering for H1 and H2 headings (true / false, default: true).
   - table_of_contents: Enable Table of Contents navigation menu (true / false, default: false).

2. DOCUMENT STRUCTURE:
   - Level 1 Headings (# Heading): Collapsible main section headers (without completion badge).
   - Level 2 Headings (## Heading): Collapsible sections with completion badges.
   - Level 3-6 Headings (### Subheading): Styled subheadings inside sections.

3. CHECKLISTS & LISTS:
   - Task items (- [ ] or - [x]): Interactive checkboxes saved in browser storage.
   - Nested task items (  - [ ] Subtask): Indented sub-checklists.
   - Bullet items (- Item): Standard bullet list entries with nesting support.
   - Ordered items (1. Item): Sequential numbered list entries.

4. CALLOUT / NOTE BOXES:
   - > Note: Neutral information panel.
   - >? Tip: Success / tip panel (green accent).
   - >! Important: Important note panel (purple accent).
   - >!! Warning: Warning alert panel (yellow accent).
   - >!!! Caution: Danger / caution panel (red accent).

5. CODE BLOCKS:
   - Fenced code blocks (```lang ... ```) render with language tags and copy button.

6. TABLES & CODE VARIABLES:
   - Standard GFM tables (| Column 1 | Column 2 |) render as responsive data tables.
   - Dynamic variables table ([Variables]) defines key-value pairs replaced in code blocks via {{VARIABLE_NAME}}.

7. IMAGES & LINKS:
   - Local images (![Alt text](./path/to/image.png)) are automatically embedded as Base64.
   - Remote images (![Alt text](https://example.com/image.png)) are preserved as remote <img> tags.
   - Non-image files (![Doc](./manual.pdf)) are automatically converted to external <a> links.
   - Hyperlinks ([Link text](https://example.com)) render as clickable links.

8. TEXT FORMATTING:
   - Bold (**bold text**) and strikethrough (~~deleted text~~) styling.

9. COMMENTS:
   - HTML comments (like this) are ignored during parsing and omitted from output HTML.

10. DATE SHORTCUTS & INTERACTIVE INPUTS:
   - Type "today" into any date input field to automatically insert today's date.
===============================================================================
-->
---
title: "Doc2Flow Standard Operating Procedure"
subtitle: "Interactive Setup & Maintenance Protocol"
version: "1.0.0"
date: "2026-07-25"
language: "en"
numbered_sections: true
table_of_contents: false
---

[Variables]
| Variable | Value |
| --- | --- |
| TARGET_HOST | 192.168.1.100 |
| SERVICE_PORT | 8080 |

# Part 1: System Setup & Preparation

This top-level section defines the overall operational scope. It replaces ~~legacy manual procedures~~ with **modern automated verification workflows**.

## Section 1: Initial System Verification

<!-- Verification of baseline prerequisites -->

### Prerequisites Checklist

- [ ] Check operating system compatibility and network connectivity
  - [ ] Verify primary network interface (eth0 / IPv4)
  - [x] Test DNS resolution and gateway reachability
- [x] Verify administrator credentials and execution permissions
- [ ] Confirm target directory layout and storage capacity

### Execution Steps

1. Review system architecture diagrams and prerequisites.
2. Initialize runtime environment configuration.
3. Validate operational readiness before service deployment.

### Reference Links & Documentation

- [External System Documentation](https://example.com/docs)
- ![Download System Specification PDF](https://example.com/files/specification.pdf)

### Essential Information

> Note: All actions in this procedure are logged and stored locally in browser storage.

>? Tip: You can toggle section completion by checking off all interactive items.

# Part 2: Maintenance & Reference

## Section 2: Configuration & Service Deployment

### Environment Setup

>! Important: Ensure configuration settings are validated before starting services.

Execute the setup command:

```bash
# Initialize system configuration
d2f --init custom_guide.md --host {{TARGET_HOST}} --port {{SERVICE_PORT}}
```

>!! Warning: Modifying environment settings will cause a service restart.

>!!! Caution: Do not delete existing database volumes without a verified backup.

### System Infrastructure Matrix

| Component | Endpoint | Status |
| --- | --- | --- |
| Control Service | 192.168.1.100:8080 | Active |
| Database Node | 192.168.1.101:5432 | Ready |
| Worker Agent | 192.168.1.102:9090 | Standby |

- Standard bullet item 1: Verify log directory permissions
  - Sub-item A: Ensure log rotation is active
  - Sub-item B: Check disk quota limits
- Standard bullet item 2: Backup system configuration before final sign-off
