<!--
===============================================================================
DOC2FLOW (D2F) v{{APP_VERSION}} - TEMPLATE & USAGE GUIDE
Repository: https://github.com/akjir/doc2flow
License: GPL-3.0-or-later
===============================================================================

1. FRONTMATTER METADATA:
   - title: Main document title.
   - subtitle: Subtitle or document description.
   - company: Company organization name.
   - contact: Responsible contact person name.
   - agent: Executing agent name.
   - date: Protocol date (YYYY-MM-DD).
   - version: Document version.
   - language / lang: Language code for static UI translation ('en', 'de').
   - logo: Optional path to a custom logo image (SVG, PNG, JPG, WebP).

2. DOCUMENT STRUCTURE:
   - Level 1 Headings (# Heading): Define non-collapsible section headers.
   - Level 2 Headings (## Heading): Define collapsible sections with progress badges.
   - Level 3 Headings (### Subheading): Define subheadings within sections.

3. CHECKLISTS & BULLETS:
   - Task items (- [ ] or - [x]): Interactive checkboxes saved in browser localStorage.
   - Bullet items (- Item): Standard bullet list entries.

4. CALLOUT / NOTE BOXES:
   - > Note: Neutral information panel.
   - >? Tip: Success / tip panel (green accent).
   - >! Important: Important note panel (blue accent).
   - >!! Warning: Warning alert panel (orange accent).
   - >!!! Caution: Danger / caution panel (red accent).

5. CODE BLOCKS:
   - Fenced code blocks (```lang ... ```) render with language tags and copy button.

6. IMAGES & LINKS:
   - Local images (![Alt text](./path/to/image.png)) are automatically embedded as Base64.
   - Remote images (![Alt text](https://example.com/image.png)) are preserved as remote <img> tags.
   - Non-image files (![Doc](./manual.pdf)) are automatically converted to external <a> links.
   - Hyperlinks ([Link text](https://example.com)) render as clickable links.

7. COMMENTS:
   - HTML comments (like this) are ignored during parsing and omitted from output HTML.

8. DATE SHORTCUTS & INTERACTIVE INPUTS:
   - Type "today" into any date input field to automatically replace it with today's date (formatted via date_placeholder or browser locale).
===============================================================================
-->
---
title: "Doc2Flow Standard Operating Procedure"
subtitle: "Interactive Setup & Maintenance Protocol"
company: "Acme Corporation"
contact: "Jane Doe"
agent: "John Smith"
version: "1.0.0"
date: "2026-07-25"
language: "en"
---

## Section 1: Initial System Verification

<!-- Verification of baseline prerequisites -->

### Prerequisites Checklist

- [ ] Check operating system compatibility and network connectivity
- [x] Verify administrator credentials and execution permissions
- [ ] Confirm target directory layout and storage capacity

### Reference Links & Documentation

- [External System Documentation](https://example.com/docs)
- ![Download System Specification PDF](https://example.com/files/specification.pdf)

### Essential Information

> Note: All actions in this procedure are logged and stored locally.

>? Tip: You can toggle section completion by checking off all interactive items.

## Section 2: Configuration & Service Deployment

### Environment Setup

>! Important: Ensure configuration settings are validated before starting services.

Execute the setup command:

```bash
# Initialize system configuration
d2f --init custom_guide.md
```

>!! Warning: Modifying environment settings will cause a service restart.

>!!! Caution: Do not delete existing database volumes without a verified backup.

- Standard bullet item 1: Verify log directory permissions
- Standard bullet item 2: Backup system configuration before final sign-off
