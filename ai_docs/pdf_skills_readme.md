# PDF Skills - Three-Tier Architecture

This directory contains three specialized PDF skills organized in a hierarchical architecture.

## Quick Reference

| Skill | Use When | Focus |
|-------|----------|-------|
| `/pdf-spec` | "What does the spec require?" | ISO 32000-1:2008 normative requirements |
| `/pdf-debugging` | "Why does this PDF fail?" | Real-world parsing failures and repairs |
| `/pdf-tools` | "How do I manipulate PDFs externally?" | Python/CLI tools (pypdf, qpdf, etc.) |

## Architecture

```
┌─────────────────────────────────────────────┐
│  /pdf-spec                                  │
│  "The Source of Truth"                      │
│  • ISO 32000 specification reference        │
│  • Required vs optional fields              │
│  • Algorithm descriptions (normative)       │
└─────────────────────────────────────────────┘
                    ▲
                    │ references
                    │
┌─────────────────────────────────────────────┐
│  /pdf-debugging                             │
│  "The Implementation Reality"               │
│  • Malformed PDF diagnosis                  │
│  • Parser failure patterns                  │
│  • Repair strategies                        │
└─────────────────────────────────────────────┘

┌─────────────────────────────────────────────┐
│  /pdf-tools                                 │
│  "The External Ecosystem"                   │
│  • Python libraries (pypdf, pdfplumber)     │
│  • CLI tools (qpdf, pdftotext)              │
│  • Test fixture generation                  │
└─────────────────────────────────────────────┘
```

## Workflow Patterns

### Pattern 1: Implementing a New PDF Feature
1. **Start with `/pdf-spec`** - Understand ISO requirements
2. **Write implementation** - Follow spec closely
3. **Handle edge cases with `/pdf-debugging`** - Add repair strategies for real-world deviations

### Pattern 2: Fixing a Parsing Failure
1. **Use `/pdf-debugging`** - Diagnose the issue
2. **Reference `/pdf-spec`** - Verify what the spec requires
3. **Optionally use `/pdf-tools`** - Cross-validate with pypdf/qpdf

### Pattern 3: Creating Test Fixtures
1. **Use `/pdf-tools`** - Generate PDFs with specific features using Python
2. **Verify with `/pdf-spec`** - Ensure compliance
3. **Test parsing** - Use oxidize-pdf, debug with `/pdf-debugging` if needed

## Deduplication Strategy

**Encryption Example:**
- `/pdf-spec` contains: Algorithm 2, 2.A, 2.B descriptions, required dictionary entries
- `/pdf-debugging` contains: Password validation failures, encoding issues, triage steps
- No overlap - spec is normative, debugging is practical

**Font Example:**
- `/pdf-spec` contains: Type0/Type1 required fields, descriptor requirements
- `/pdf-debugging` contains: Missing descriptor fallbacks, circular reference detection
- No overlap - spec defines structure, debugging fixes violations

## Cross-References

Each skill explicitly references the others:

- **pdf-spec** → References pdf-debugging for handling spec violations
- **pdf-debugging** → References pdf-spec for normative requirements, pdf-tools for validation
- **pdf-tools** → References pdf-spec for compliance, pdf-debugging for troubleshooting

## Migration Notes

**From old structure (2026-01-24):**
- `pdf-iso-expert` → `pdf-spec` (renamed for clarity)
- `pdf` → `pdf-tools` (renamed to indicate external tools)
- `pdf-debugging` → No change (already well-named)

All skills updated with "When to Use This Skill" sections for better discoverability.
