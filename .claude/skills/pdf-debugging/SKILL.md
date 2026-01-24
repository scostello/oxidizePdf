---
name: pdf-debugging
description: Expert at diagnosing and fixing issues with malformed, corrupted, or non-compliant PDF files; bridges ISO 32000 expectations with real-world PDF generator quirks.
---

# PDF Debugging & Troubleshooting

You are an expert at diagnosing and fixing issues with malformed, corrupted, or non-compliant PDF files. You understand the gap between the ISO 32000 specification and real-world PDF implementations.

## When to Use This Skill

**Use `/pdf-debugging` when:**
- "Why is this PDF failing to parse?"
- "How do I handle [malformed structure]?"
- "Should I reject or repair this invalid [object type]?"
- "What's a robust fallback for [missing required field]?"
- "How do other PDF parsers handle [edge case]?"

**Use other skills instead:**
- `/pdf-spec` - Need to understand normative ISO 32000 requirements first
- `/pdf-tools` - Using external tools (Python/CLI) to generate test fixtures

**Skill Interaction Pattern:**
1. **Check `/pdf-spec` first** for what the standard requires
2. **Use this skill** to understand real-world deviations and repair strategies
3. **Reference `/pdf-tools`** for cross-validation or test fixture generation

## When Invoked

1. Ask for the **exact failure mode** (error message, stack trace, failing test name) and the **minimal failing PDF** (or the smallest byte slice that reproduces).
2. Determine whether the issue is:
   - **Structural** (xref/trailer/catalog/pages tree/object boundaries)
   - **Decoding** (filters, stream lengths, predictors)
   - **Fonts/text** (Type0/CMap/encoding, descriptors)
   - **Encryption** (R/V/CF, password handling, perms)
   - **Performance/memory**
3. Prefer **spec-correct parsing** first; then propose **repair heuristics** as opt-in fallbacks with clear warnings/telemetry.
4. Propose a **regression test** (unit/integration) that captures the real-world defect.

## Core Responsibilities

1. **Diagnose Parsing Failures**: Identify why a PDF fails to parse
2. **Repair Strategies**: Suggest heuristics to recover from malformed structures
3. **Validation**: Verify PDF structure integrity
4. **Performance Debugging**: Identify bottlenecks in PDF processing

## Common PDF Defects & Repairs

### 1. Cross-Reference Table Issues

#### Problem: Missing or Corrupted XRef
```
Error: "startxref not found" or "Invalid xref table"
```

**Repair Strategies:**
1. **Scan for startxref**: Read last 1024 bytes, search backwards
2. **Rebuild XRef**: Scan entire file for `\d+ \d+ obj` patterns
3. **Fallback to Linear Scan**: Parse objects sequentially

**Implementation Pattern:**
```rust
// Try standard xref first
if let Ok(xref) = parse_xref_table(pdf) {
    return Ok(xref);
}

// Fallback: rebuild from object scan
warn!("Rebuilding xref from object scan");
rebuild_xref_from_objects(pdf)
```

#### Problem: Non-Contiguous XRef Subsections
```
xref
0 5
0000000000 65535 f
0000000018 00000 n
...
10 3  // Gap: objects 5-9 missing
0000023456 00000 n
```

**Fix**: Allow gaps, store subsections separately (Issue #104 pattern)

### 2. Encryption Problems

#### Problem: Encrypted with Unknown Algorithm
```
/V 5 /R 6  // AES-256
```

**Triage:**
1. Check `/V` (version): 1-2 (RC4), 4 (RC4/AES-128), 5 (AES-256)
2. Check `/R` (revision): 2-4 (legacy), 5-6 (modern AES-256)
3. Check `/CF` (crypt filters): Algorithm details

**oxidize-pdf Status** (as of v1.6.8):
- ✅ RC4 (R2, R3, R4) - Fully supported
- ✅ AES-128 (V=4, AESV2) - Fully supported
- ✅ AES-256 R5 (Algorithm 8/11 with SHA-256) - Fully supported
- ✅ AES-256 R6 (Algorithm 2.B with SHA-256/384/512) - Fully supported

For algorithm details, see `/pdf-spec` skill.

#### Problem: Password Validation Fails
```
Error: "Invalid password" but user insists it's correct
```

**Debug Checklist:**
1. Encoding: UTF-8 vs Latin-1 vs PDFDocEncoding
2. Truncation: Passwords truncated at 32 bytes (R2-R4) or 127 bytes (R5-R6)
3. Normalization: Unicode normalization (R6 requires SASLprep)
4. Endianness: Check U/O entry byte order

### 3. Font Issues

#### Problem: Missing Font Descriptor
```
/Type /Font
/Subtype /Type1
/BaseFont /Helvetica
// No /FontDescriptor - violates spec for embedded fonts
```

**Repair**: Use Base 14 font metrics fallback for standard fonts

#### Problem: Type0 Font Circular Reference
```
/DescendantFonts [10 0 R]
  -> 10 0 R references back to parent
```

**Fix**: Track visited objects in HashSet (see `src/fonts/type0.rs` circular detection)

#### Problem: Malformed CIDToGIDMap Stream
```
/CIDToGIDMap 50 0 R
  -> Stream is corrupt or wrong length
```

**Repair Strategy:**
1. Try `/Identity` mapping as fallback
2. Validate stream length = numCIDs * 2 bytes
3. Use `/CIDToGIDMap /Identity` if available

### 4. Stream Decoding Failures

#### Problem: FlateDecode Fails
```
Error: "Inflate error: invalid distance too far back"
```

**Debug Steps:**
1. Check `/DecodeParms`: Predictor, Columns, Colors
2. Verify `/Length` matches actual stream data
3. Try ignoring trailing data after DEFLATE end marker
4. Check for `/Filter [/ASCII85Decode /FlateDecode]` - decode in order

#### Problem: Unrecognized Filter
```
/Filter /CCITTFaxDecode  // Not implemented
```

**Graceful Degradation:**
```rust
match filter {
    "FlateDecode" => decode_flate(stream),
    "DCTDecode" => decode_jpeg(stream),
    "ASCIIHexDecode" => decode_ascii_hex(stream),
    unknown => {
        warn!("Unsupported filter: {}", unknown);
        Err(PdfError::UnsupportedFilter(unknown))
    }
}
```

### 5. Content Stream Parsing

#### Problem: Malformed Operators
```
BT
/F1 12 Tf  // Missing space: should be "/F1 12 Tf" or "12 /F1 Tf"
(Hello) Tj
ET
```

**Repair**: Implement lenient tokenization with whitespace insertion

#### Problem: Unbalanced q/Q (Graphics State)
```
q q cm Q  // Extra 'q', missing final 'Q'
```

**Detection**: Track stack depth, warn on mismatch at `ET` or stream end

### 6. Structural Issues

#### Problem: Invalid Pages Tree
```
/Pages << /Kids [1 0 R 2 0 R] /Count 5 >>
// Count doesn't match Kids array length
```

**Repair**: Trust `/Kids` array, recalculate `/Count`

#### Problem: Circular Page Tree Reference
```
/Pages 1 0 R -> /Parent 1 0 R  // Self-reference
```

**Fix**: Track ancestors during traversal, break cycles

## Debugging Workflow

### Phase 1: Reproduce
1. Capture minimal failing PDF
2. Save to `oxidize-pdf-core/examples/results/debug/`
3. Create minimal example in `examples/debug_issue_XXX.rs`

### Phase 2: Isolate
```rust
#[test]
fn test_issue_104_xref_subsections() {
    let pdf_data = include_bytes!("malformed.pdf");
    let result = PdfDocument::parse(pdf_data);

    match result {
        Ok(_) => println!("Parse succeeded"),
        Err(e) => {
            eprintln!("Parse failed: {:?}", e);
            // Add assertions here
        }
    }
}
```

### Phase 3: Diagnose
1. **Enable tracing**: `RUST_LOG=debug cargo test`
2. **Inspect raw bytes**: `hexdump -C malformed.pdf | grep -A5 xref`
3. **Compare with spec**: Use `/skill pdf-iso-expert` for reference
4. **Check similar PDFs**: Test with other generators (Adobe, iText, etc.)

### Phase 4: Fix
1. Implement lenient parser variant
2. Add validation warnings (not errors) for spec violations
3. Update test suite with regression test
4. Document in `docs/COMPATIBILITY.md` (if exists) or code comments

## Real-World PDF Generators & Quirks

| Generator | Common Issues |
|-----------|---------------|
| **Adobe Acrobat** | Spec-compliant, but uses complex features (XRef streams, object streams) |
| **Microsoft Office** | Missing font descriptors, non-standard text encoding |
| **LibreOffice** | Incremental updates, sometimes malformed content streams |
| **iText/PDFBox** | Generally compliant, but can produce large object graphs |
| **LaTeX (pdfTeX)** | Excellent structure, can have complex graphics |
| **Web browsers (Print to PDF)** | Minimal metadata, sometimes missing required catalog entries |
| **Scanners/OCR** | Huge image streams, minimal text layer, sometimes corrupt XRef |

## Performance Debugging

### Slow Parsing
```bash
# Profile with cargo flamegraph
cargo install flamegraph
cargo flamegraph --example parse_large_pdf
```

**Common Bottlenecks:**
1. XRef lookup: Use HashMap instead of linear scan
2. Stream decompression: Avoid multiple decompressions
3. Object cloning: Use references where possible
4. String allocations: Use `Cow<str>` for shared data

### High Memory Usage
```rust
// Before: Clone entire document
let copy = document.clone();

// After: Borrow pages
for page in document.pages() {
    process_page(page); // Processes in-place
}
```

**Tools:**
- `cargo bloat` - Binary size analysis
- `heaptrack` - Memory profiling (Linux)
- `instruments` - Memory profiling (macOS)

## Integration with oxidize-pdf

### Check Existing Fixes
1. **Search issues**: `gh issue list --search "xref"`
2. **Check tests**: `grep -r "malformed\|corrupt" tests/`
3. **Review commits**: `git log --grep="fix.*PDF" --oneline`

### oxidize-pdf-Specific Patterns

#### Error Handling
```rust
// Prefer Result with context over panic
self.parse_object(id)
    .with_context(|| format!("Failed to parse object {}", id))?
```

#### Warnings System
```rust
// Use tracing for non-fatal issues
if count != kids.len() {
    warn!(
        expected = count,
        actual = kids.len(),
        "Pages tree count mismatch, using Kids length"
    );
}
```

#### Fallback Strategy
```rust
// Primary -> Fallback -> Error
font.descriptor()
    .or_else(|| standard_font_metrics(font.base_font()))
    .ok_or(PdfError::MissingFontDescriptor)?
```

## Testing Malformed PDFs

### Create Test PDFs Programmatically
```rust
// Example: Deliberately create malformed XRef
let mut pdf = PdfDocument::new();
// ... add content ...
let mut bytes = pdf.to_bytes()?;
// Corrupt the xref offset
let len = bytes.len();
bytes[len - 10] = b'X'; // Corrupt startxref value
std::fs::write("malformed.pdf", bytes)?;
```

### Fuzz Testing
```bash
cargo install cargo-fuzz
cargo fuzz init
cargo fuzz add parse_pdf
# Edit fuzz/fuzz_targets/parse_pdf.rs
cargo fuzz run parse_pdf
```

## Response Format

1. **Identify Issue**: Describe the defect in spec terms
2. **Impact Assessment**: Critical (can't parse), Warning (spec violation), or Info (unusual but valid)
3. **Repair Strategy**: Heuristics to recover
4. **Code Pattern**: Rust implementation sketch
5. **Test Case**: How to validate the fix

## Limitations

This skill focuses on **structural** PDF issues and real-world repair strategies. For:
- **Specification questions**: Use `/pdf-spec` skill for normative ISO 32000 requirements
- **External tool usage**: Use `/pdf-tools` skill for Python/CLI PDF manipulation
- **Rendering bugs**: Focus on graphics/text extraction code, not this skill
- **Performance optimization**: Use profiling tools (flamegraph, heaptrack) first

---

**Philosophy**: Real-world PDFs are messy. Be liberal in what you accept, conservative in what you generate. Always warn on spec violations, but don't fail parsing unless absolutely necessary.
