# oxidize-pdf Project Memory

## Current Status (2026-01-24)

**Branch**: `chore/curate-claude-skills`
**Version**: v1.6.8 (released to crates.io)
**Quality**: Grade A (95/100)

### Key Metrics
| Metric | Value |
|--------|-------|
| Tests | 5005 unit + 185 doc tests ✅ |
| Coverage | 70.00% |
| PDF Success Rate | 99.3% (275/277 PDFs) |
| ISO Compliance | 310 curated requirements, 100% linked |
| Performance | 5,500-6,034 pages/sec |

---

## Recent Work

### Session 2026-01-24: Claude Skills Curation
**Focus**: Curating and documenting Claude Code skills
- Updated CLAUDE.md with comprehensive skills reference (PDF, Rust, GUI, Graphics, Utility)
- Updated memory.md skills section to match
- Organized skills into categories: PDF, Rust Development, GUI Development, Graphics & Utility
- All skills documented in `.claude/skills/` directory

**Branch**: `chore/curate-claude-skills`
**Status**: Complete

### Session 2026-01-17: Issue #115 Font Subsetting Fix
**Problem**: Large fonts (41MB CJK) with few characters (4 chars) produced 41MB PDFs with no subsetting.

**Root Cause**: `truetype_subsetter.rs` skipped subsetting when `char_count < 10`, ignoring font size.

**Solution**:
- Added `should_skip_subsetting(font_size, char_count)` function
- New thresholds: `SUBSETTING_SIZE_THRESHOLD` (100KB), `SUBSETTING_CHAR_THRESHOLD` (10)
- Rule: Large fonts (>100KB) ALWAYS subset, regardless of character count
- 9 new TDD tests covering edge cases

**Files Modified**: `oxidize-pdf-core/src/text/fonts/truetype_subsetter.rs`

### Session 2026-01-10: v1.6.8 Release
- Fixed Cargo.toml version desync (1.6.6 → 1.6.8)
- Added pypdf encryption test fixtures to git
- Excluded test fixtures from crates.io package (8.9MB → under 10MB limit)
- Updated README.md dependency versions
- Fixed gitflow workflow (develop_santi → develop → main → tag)

### Session 2026-01-05: Encryption Complete ✅
**All encryption features now production-ready:**
- ✅ AES-256 R5 support (Algorithm 8/11) - 9 real PDF tests
- ✅ AES-256 R6 support (Algorithm 2.B) - 10 real PDF tests
- ✅ Owner password R5/R6 - 15 tests
- ✅ Cross-validation with pypdf - 6 tests passing
- ✅ Performance benchmarks: R5 (862ns), R6 (1.78ms), RC4 (30µs)

**Total encryption tests**: 302+ including 19 real PDF integration tests (qpdf compatible)

---

## Critical Features Completed

### ✅ Font Subsetting (v1.6.8)
- Size-aware subsetting for CJK and large fonts
- TrueType subsetting with glyph remapping
- Type0/CID font full embedding support

### ✅ Encryption (v1.6.5-1.6.8)
- RC4 (40-bit, 128-bit)
- AES-128, AES-256 (R5, R6)
- User and owner password support
- Algorithm 2.B (ISO 32000-2:2020) with SHA-256/384/512
- qpdf and pypdf compatibility verified

### ✅ Text Extraction
- Position-aware fragments
- Kerning normalization (Issue #87)
- LLM-optimized formats
- Invoice data extraction (custom API)

### ✅ Parser Robustness
- XRef non-contiguous subsections (Issue #104)
- Linearized PDF support (Issue #98)
- UTF-8 panic fix with byte-based recovery (Issue #93)
- 99.3% success rate on failure corpus

---

## Architecture

```
oxidize-pdf/
├── oxidize-pdf-core/    # Core PDF library (main crate)
├── oxidize-pdf-api/     # REST API server
├── oxidize-pdf-cli/     # Command-line interface
└── oxidize-pdf-render/  # Rendering engine (separate repo)
```

### Critical Files Map

| Component | File | Lines | Purpose |
|-----------|------|-------|---------|
| Parser | `parser/document.rs` | 1886 | PdfDocument API |
| Writer | `writer/pdf_writer.rs` | 3912 | PDF generation |
| Graphics | `graphics/pdf_image.rs` | 2006 | Image handling |
| OCR | `text/ocr.rs` | 1879 | Text recognition |
| Encryption | `security/standard_security.rs` | ~1500 | R2-R6 algorithms |
| Fonts | `text/fonts/truetype_subsetter.rs` | ~800 | Subsetting logic |

---

## Development Rules

### Critical Policies
- **Treat all warnings as errors** (clippy + rustc)
- **Minimum 80% test coverage** (target 95%)
- **NO manual releases** - GitHub Actions pipeline only
- **ALL PDFs** → `oxidize-pdf-core/examples/results/`
- **NO unwraps** in library code (51 eliminated in v1.6.2)

### Commands
```bash
cargo test --workspace          # All tests
cargo clippy -- -D warnings     # Linting
cargo fmt --all --check         # Formatting
cargo run --example <name>      # Examples
```

### Git Workflow
1. Work on `develop_santi` branch
2. PR to `develop` → merge to `main`
3. Tag releases on `main` (triggers CI/CD)

---

## Known Limitations

| Issue | Impact | Status |
|-------|--------|--------|
| 2 malformed PDFs | VERY LOW | Genuine format violations |
| CID/Type0 fonts | NONE | ✅ Resolved (Phase 3.4) |
| AES-256 R5/R6 | NONE | ✅ Resolved (v1.6.5-1.6.8) |
| Owner passwords | NONE | ✅ Resolved (v1.6.8) |

---

## Test Coverage Lessons

### What Works
1. **Pure logic modules** - Math, transformations, parsers
2. **Small modules** - <200 lines, 30-85% current coverage
3. **Targeted tests** - Assert specific code paths, not just `is_ok()`

### What Doesn't Work
1. **API-only tests** - Testing public API doesn't improve coverage
2. **Smoke tests** - `assert!(result.is_ok())` is useless
3. **I/O modules** - Requires real PDFs/fonts, low ROI

**Example Success**: `coordinate_system.rs` - 51 lines, 0%→100% coverage (pure math)
**Example Failure**: `parser/reader.rs` - 42 tests, 0% improvement (API already covered)

---

## PDF Editor Project (Separate Location)

**Status**: Phase 1 complete, Phase 2 starting
**Location**: `/Users/scostello/Development/projects/projects-learning/reference-projects/pdf-internals/oxidizePdf/oxidize-pdf-editor/`
**Tech Stack**: iced 0.14.0 + rfd 0.17.2 + oxidize-pdf v1.6.8

### ✅ Phase 1 Complete (Basic Viewer)
- ✅ File dialog integration (rfd)
- ✅ PDF loading with metadata display
- ✅ Page navigation (prev/next with boundary checking)
- ✅ Tokyo Night Storm theme
- ✅ Error handling and loading states

### 🔄 Phase 2 In Progress (Bookmark Extraction)
**Goal**: Display PDF document outline/bookmarks in a sidebar panel
**Requirements**:
- Parse PDF outline dictionary (`/Outlines`, `/Outline` entries)
- Extract hierarchical bookmark structure (title, destination, children)
- Display in collapsible tree view (iced widget)
- Handle bookmark clicks to navigate to target page
- Support nested bookmark levels

**Core API**: Will use `oxidize-pdf-core` outline parsing (needs implementation check)

### 📋 Future Phases (3-8)
3. Thumbnail generation (lightweight preview)
4. Multi-document side-by-side view
5. Pan & zoom
6. Annotation reading
7. Page editing (delete, rotate, reorder)
8. Annotation writing

**Note**: Editor uses iced 0.14.0 builder pattern, not trait-based Application

---

## Sprint History

| Sprint | Grade | Achievement |
|--------|-------|-------------|
| Sprint 1 | A- (90) | Code hygiene: 171 prints migrated, tracing infrastructure |
| Sprint 2 | A (93) | Performance: 91 clones removed, 10-20% memory savings |
| Sprint 3 | C (67) | CI: pre-commit hooks; Coverage task failed (lesson learned) |
| Sprint 4 | A (95) | ISO Matrix: 7,775→310 curated requirements, auto-linking |

**Sprint 3 Lesson**: API coverage ≠ code coverage. Need HTML report inspection for targeted tests.

---

## Next Priorities

1. **PDF Editor Phase 2** - Bookmark extraction (outline parser) - IMMEDIATE FOCUS
2. Continue coverage improvement (70% → 80%)
3. Investigate remaining 2 malformed PDFs
4. Performance profiling for large documents
5. ISO compliance gap analysis (310 requirements, 55-60% implemented)

---

## Claude Code Configuration

### Skills Available
Located in `.claude/skills/` directory:

**PDF Skills (Three-Tier Architecture):**
- `/pdf-spec` - ISO 32000-1:2008 specification reference (normative source of truth)
- `/pdf-debugging` - Malformed PDF diagnosis and repair strategies
- `/pdf-tools` - External tools: pypdf, pdfplumber, qpdf, reportlab

**Rust Development Skills:**
- `/rust-engineer` - Rust patterns, ownership, async, traits, error handling
- `/rust-perf-expert` - Performance optimization, profiling, benchmarking

**GUI Development Skills:**
- `/iced-dev` - Building iced GUI applications (architecture, widgets, styling)
- `/iced-testing` - Testing iced apps (unit tests, headless UI tests, CI/CD)

**Graphics & Utility Skills:**
- `/graphics-core` - Graphics math, coordinate transforms, color spaces, curves
- `/progressive-file-disclosure` - Handling large files efficiently
- `/skill-creator` - Creating or updating Claude Code skills

### Memory System
- **Project Memory**: `.claude/memory.md` (this file)
- **Settings**: `.claude/settings.local.json`
- **Purpose**: Maintains context across sessions, tracks progress, documents decisions

---

## External Resources
- **GitHub**: https://github.com/bzsanti/oxidizePdf
- **Crates.io**: https://crates.io/crates/oxidize-pdf
- **Docs**: `docs/ARCHITECTURE.md`, `docs/INVOICE_EXTRACTION_GUIDE.md`, `docs/LINTS.md`
- **Roadmap**: `.private/ROADMAP_MASTER.md` (confidential)
