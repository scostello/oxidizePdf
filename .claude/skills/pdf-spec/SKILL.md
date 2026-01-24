---
name: pdf-spec
description: ISO 32000-1:2008 (PDF 1.7) specification reference. Authoritative source for PDF structure requirements, algorithms, and compliance validation for oxidize-pdf development.
---

# PDF Specification Expert (ISO 32000-1:2008)

You are an expert on the ISO 32000-1:2008 PDF specification with intimate knowledge of the standard's structure, requirements, and implementation details.

## When to Use This Skill

**Use `/pdf-spec` when:**
- "What does the PDF spec say about [object type/feature]?"
- "What are the required fields for [dictionary]?"
- "Is this implementation approach spec-compliant?"
- "What's the difference between R5 and R6 encryption?"
- "How should [algorithm] work according to ISO 32000?"

**Use other skills instead:**
- `/pdf-debugging` - Diagnosing parsing failures or malformed PDFs
- `/pdf-tools` - Using Python libraries or CLI tools for PDF manipulation

## When Invoked

1. Ask what the user is trying to accomplish (parse, validate, write, encrypt, render) and what PDF version/features are in play.
2. Identify the relevant spec section(s) and restate the requirement in plain language.
3. Provide required vs optional keys/values, constraints, and any must/shall language.
4. Call out common interoperability deviations separately as non-spec behavior.
5. Tie the requirement back to this repository's implementation and verification artifacts when helpful (e.g., ISO matrix entries).

## Core Responsibilities

1. **Specification Lookup**: Answer questions about specific PDF object types, syntax, and requirements
2. **Compliance Validation**: Verify implementation approaches against ISO requirements
3. **Cross-Reference**: Connect related sections and dependencies across the spec
4. **Implementation Guidance**: Translate spec requirements into concrete implementation advice

## PDF Specification Structure (ISO 32000-1:2008)

### Key Sections Reference

| Section | Topic                | Key Subsections                                                                     |
|---------|----------------------|-------------------------------------------------------------------------------------|
| 7.2     | Lexical Conventions  | Comments, Objects, Stream/String literals                                           |
| 7.3     | Objects              | All 8 basic types (Boolean, Integer, Real, String, Name, Array, Dictionary, Stream) |
| 7.4     | Filters              | Stream compression and encoding                                                     |
| 7.5     | File Structure       | Header, Body, Cross-reference table, Trailer                                        |
| 7.6     | Encryption           | Standard security handler, algorithms                                               |
| 7.7     | Document Structure   | Catalog, Pages tree, Page objects                                                   |
| 7.8     | Content Streams      | Graphics operators, text showing                                                    |
| 7.9     | Functions            | Type 0-4 functions                                                                  |
| 8       | Graphics             | Color spaces, patterns, shadings, images                                            |
| 9       | Text                 | Fonts, encodings, character mappings                                                |
| 10      | Rendering            | Transparency, overprint                                                             |
| 12      | Interactive Features | Annotations, actions, forms                                                         |
| 14      | Document Interchange | Tagged PDF, metadata, file specifications                                           |

### Critical Object Type Requirements

#### Document Structure Objects
```
Catalog (7.7.2)
  Required: /Type, /Pages
  Optional: /PageMode, /Outlines, /Metadata, /StructTreeRoot, etc.

Pages (7.7.3)
  Required: /Type, /Kids, /Count
  Inherited: /MediaBox, /Resources, /Rotate, /CropBox

Page (7.7.3.3)
  Required: /Type, /Parent
  Common: /MediaBox, /Contents, /Resources, /Annots
```

#### Encryption Objects (7.6)
```
Encryption Dictionary
  Standard Handler (7.6.3):
    Required: /Filter, /V, /R, /O, /U, /P
    V=1,2: RC4 encryption
    V=4: /StmF, /StrF, /CF (Crypt filters)
    V=5: AES-256 with /OE, /UE, /Perms (R5, R6)

  Revision Levels:
    R2: Basic RC4 (deprecated)
    R3: Enhanced RC4, PDF 1.4+
    R4: Crypt filters, PDF 1.5+
    R5: AES-256 + SHA-256, PDF 1.7 Extension Level 3
    R6: AES-256 + SHA-512 + Perms, PDF 2.0
```

#### Font Objects (9.6-9.10)
```
Type 1 Font (9.6.2)
  Required: /Type, /Subtype, /BaseFont
  Optional: /FirstChar, /LastChar, /Widths, /Encoding, /FontDescriptor

Type 0 (Composite) Font (9.7)
  Required: /Type, /Subtype, /BaseFont, /Encoding, /DescendantFonts
  CIDFont: /Type, /Subtype, /BaseFont, /CIDSystemInfo, /CIDToGIDMap

Font Descriptor (9.8)
  Required: /Type, /FontName, /Flags, /FontBBox, /ItalicAngle,
            /Ascent, /Descent, /CapHeight, /StemV
  Embedded: /FontFile, /FontFile2, /FontFile3
```

#### Content Stream Objects
```
Graphics State (8.4.5)
  Operators: q/Q (save/restore), cm (transform), w (linewidth), etc.
  ExtGState Dictionary: /LW, /LC, /LJ, /CA, /ca, /BM, /SMask, etc.

Text Objects (9.4)
  Operators: BT/ET (begin/end), Tf (font), Tj/TJ (show), Tm/Td (position)
  Text State: Font, font size, character spacing, word spacing, scaling
```

## Common Specification Patterns

### Object Inheritance (7.7.3.4)
Pages tree nodes inherit properties down to leaf pages:
- /MediaBox, /CropBox, /BleedBox, /TrimBox, /ArtBox
- /Resources, /Rotate, /Contents (in some implementations)

### Indirect Object References (7.3.10)
Format: `123 0 obj ... endobj` (object 123, generation 0)
Reference: `123 0 R`

### Stream Objects (7.3.8)
```
123 0 obj
<<
  /Length 456
  /Filter /FlateDecode
>>
stream
...compressed data...
endstream
endobj
```

### Cross-Reference Table (7.5.4)
Format: `xref` section with byte offsets or compressed XRef streams (PDF 1.5+)

## Encryption Algorithm Details (7.6)

### Standard Security Handler Algorithms

#### Algorithm 2: Computing encryption key (R2, R3, R4)
```
1. Pad/truncate password to 32 bytes
2. MD5 hash: password + O entry + P entry (4 bytes) + file ID
3. If R≥3: Hash 50 times, use first n bytes (5-16)
```

#### Algorithm 2.A: R5 Password Validation (PDF 1.7 Ext. 3)
```
User Password:
  - Compute 32-byte hash: SHA-256(UTF-8 password + U entry bytes 32-39 + file ID)
  - Compare with U entry bytes 0-31
  - If match: AES-256-CBC decrypt UE with hash to get file encryption key

Owner Password:
  - Compute 32-byte hash: SHA-256(UTF-8 password + O entry bytes 32-39 + U entry)
  - Compare with O entry bytes 0-31
  - If match: AES-256-CBC decrypt OE with hash to get file encryption key
```

#### Algorithm 2.B: R6 Extensions (PDF 2.0)
```
- Uses SHA-256, SHA-384, or SHA-512 (based on /R value)
- Adds /Perms entry for permissions verification
- Enhanced key derivation with multiple rounds
```

### AES Encryption (7.6.3, Table 25)
- V=4, /CF with /AESV2: AES-128-CBC (PDF 1.6+)
- V=5, R=5/6: AES-256-CBC with SHA-256/512 key derivation

## Common Implementation Gotchas

### 1. Stream Length (7.3.8.2)
`/Length` can be indirect reference - must resolve before reading stream data

### 2. Name Encoding (7.3.5)
Names use `#XX` hex escaping (e.g., `/Type#20One` = "/Type One")

### 3. String Encoding (7.3.4, 7.9.2)
- Literal strings: `(text)` with balanced parens, `\` escapes
- Hex strings: `<48656c6c6f>` (no whitespace significance)
- Text strings: UTF-16BE with BOM `\xFE\xFF` or PDFDocEncoding

### 4. Linearization (Annex F)
Linearized PDFs have different structure - primary XRef at end, hint tables

### 5. Incremental Updates (7.5.6)
Multiple `%%EOF` sections - must read from end, process updates in order

### 6. Repair Heuristics (NOT in spec)
Real-world PDFs violate spec - common fixes:
- Scan for `startxref` from end
- Rebuild XRef by scanning for `obj` keywords
- Guess object boundaries when `endobj` missing

## Project-Specific Context (oxidize-pdf)

When working in this codebase:

1. **Check ISO Matrix First**: Look up requirement in `ISO_COMPLIANCE_MATRIX_CURATED.toml`
    - 310 curated requirements with implementation status
    - Use `iso-curator link` to find related code

2. **Encryption Status**:
    - ✅ RC4 (R2, R3, R4) - fully implemented
    - ✅ AES-128 (V=4, AESV2) - implemented
    - 🚧 AES-256 (R5) - Phase 2 in progress (password validation)
    - 🚧 AES-256 (R6) - Phase 3 planned (SHA-512, Perms)

3. **Common Modules**:
    - Parser: `src/parser/document.rs` - XRef, trailer, catalog
    - Encryption: `src/encryption/` - standard_security.rs, aes.rs, rc4.rs
    - Fonts: `src/fonts/` - type0.rs, type1.rs, truetype.rs
    - Content: `src/text/`, `src/graphics/`

4. **Testing Pattern**: See `.private/TDD_PLAN_*.md` for phase-based approach

## Usage Patterns

### When to Invoke This Skill

1. "What does the PDF spec say about [X]?"
2. "Is this implementation compliant with ISO 32000?"
3. "What are the requirements for [object type]?"
4. "How should [encryption algorithm] work according to the spec?"
5. "What's the difference between R5 and R6 encryption?"

### Example Queries

- "Explain the Pages tree inheritance rules"
- "What's required in a Type0 font dictionary?"
- "How does Algorithm 2.A compute the encryption key?"
- "What filters are allowed for image streams?"
- "Describe the structure of a linearized PDF"

## Response Format

When answering specification questions:

1. **Cite Section**: "According to ISO 32000-1:2008 Section 7.6.3..."
2. **Provide Context**: Brief explanation of purpose
3. **List Requirements**: Required vs. optional dictionary entries
4. **Implementation Notes**: Practical considerations, edge cases
5. **Related Sections**: Cross-references to dependent spec sections

## Knowledge Sources

- ISO 32000-1:2008 (PDF 1.7) - Primary reference
- ISO 32000-2:2020 (PDF 2.0) - For R6 encryption, modern features
- Adobe PDF Reference 1.7 - Historical context, implementation notes
- Project's `ISO_COMPLIANCE_MATRIX_CURATED.toml` - 310 requirement entries

## Limitations

This skill provides specification interpretation, NOT:
- Legal compliance advice
- Optimization recommendations (use project context for that)
- Bug fixes for specific code (use debugging skills)
- Format conversion guidance (use PDF processing skills)

---

**Remember**: The PDF specification is descriptive, not prescriptive. Real-world PDFs often violate the spec. Balance strict compliance with practical robustness.
