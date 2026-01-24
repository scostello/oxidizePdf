---
name: progressive-file-disclosure
description: >
  Pattern for efficiently handling large files (PDFs, images, documents, data exports)
  that exceed context window capacity. Use when working with file attachments, document
  comparison workflows, ticket systems with attachments, or any scenario requiring
  selective file content access. Provides a four-step approach - metadata-first loading,
  optional preloading, on-demand file tools (load/peek/extract), and decision guidance
  for optimal tool selection.
---

# Progressive Disclosure for Large Files

Handle files larger than context capacity by loading metadata first, then accessing content on-demand.

## Core Strategy

```
1. Metadata First   → File info without content (ID, name, size, type)
2. Optional Preload → First N KB for quick-reference files
3. On-Demand Tools  → load_file, peek_file, extract_file
4. Decision Guide   → When to use each tool
```

## Tool Selection

| Scenario | Tool | Why |
|----------|------|-----|
| Need full content | `load_file(id)` | Complete file access |
| Check file structure/header | `peek_file(id, 0, 4096)` | First 4KB reveals format |
| Find specific section | `peek_file(id, start, stop)` | Targeted byte range |
| PDF/DOCX/PPTX text analysis | `extract_file(id)` | 100x smaller than original |
| Compare documents | `extract_file` both | Text comparison feasible |

## Decision Tree

```
Is file small (<50KB)?
├─ Yes → load_file (minimal cost)
└─ No → What do you need?
        ├─ Full binary content → load_file
        ├─ Just the text/tables → extract_file
        ├─ Specific byte range → peek_file
        └─ Unknown → peek_file(0, 4096) first, then decide
```

## Implementation Patterns

### Pattern 1: Document Comparison

```
Given: file_a.pdf (2MB), file_b.pdf (3MB)
Task: Compare content

Approach:
1. extract_file(file_a) → text_a (~20KB)
2. extract_file(file_b) → text_b (~30KB)
3. Compare text_a and text_b in context
```

### Pattern 2: Ticket Attachment Triage

```
Given: 5 attachments on support ticket
Task: Find relevant information

Approach:
1. Review metadata (names, sizes, types)
2. Prioritize by relevance to ticket
3. extract_file for documents, peek_file for logs
4. load_file only if binary analysis needed
```

### Pattern 3: Large Log Analysis

```
Given: server.log (500MB)
Task: Find errors from yesterday

Approach:
1. peek_file(id, -100000, -1) → Last 100KB
2. If timestamps visible, calculate byte offset for target date
3. peek_file(id, calculated_start, calculated_end)
```

## PDF-Specific Chunking

Large PDFs benefit from page-based progressive disclosure. Use with `/pdf-tools` for implementation.

### PDF Metadata First

```python
from pypdf import PdfReader
reader = PdfReader("large_doc.pdf")

metadata = {
    "pages": len(reader.pages),
    "outline": reader.outline,      # TOC/bookmarks
    "info": reader.metadata,        # Title, author, etc.
    "size_mb": path.stat().st_size / 1_000_000
}
# Decision: 500 pages → need chunking strategy
```

### Chunking Strategies

| Strategy | When to Use | Implementation |
|----------|-------------|----------------|
| **By TOC** | Document has bookmarks | Extract sections via outline |
| **By Page Range** | No TOC, sequential read | Pages 1-20, 21-40, etc. |
| **By Search** | Looking for specific topic | Find pages with keyword |
| **By Structure** | Forms, tables, headers | pdfplumber layout analysis |

### Pattern: TOC-Based Extraction

```python
# Extract only the "Installation" chapter (pages 45-52)
import pdfplumber

def extract_section(pdf_path, start_page, end_page):
    with pdfplumber.open(pdf_path) as pdf:
        return "\n".join(
            pdf.pages[i].extract_text() or ""
            for i in range(start_page - 1, end_page)
        )

# ~8 pages of relevant context instead of 500
installation_text = extract_section("manual.pdf", 45, 52)
```

### Pattern: Keyword Search + Context

```python
def find_relevant_pages(pdf_path, keyword, context_pages=1):
    """Find pages containing keyword, include surrounding pages."""
    with pdfplumber.open(pdf_path) as pdf:
        relevant = set()
        for i, page in enumerate(pdf.pages):
            if keyword.lower() in (page.extract_text() or "").lower():
                # Add page and neighbors
                for j in range(max(0, i - context_pages),
                               min(len(pdf.pages), i + context_pages + 1)):
                    relevant.add(j)
        return sorted(relevant)

# Find pages about "authentication" with 1 page context each side
pages = find_relevant_pages("api_docs.pdf", "authentication")
# Returns: [23, 24, 25, 67, 68, 69] instead of all 200 pages
```

### Pattern: Progressive Page Loading

```
Given: technical_spec.pdf (300 pages, 15MB)
Task: Answer question about "error handling"

Approach:
1. Get TOC → identify relevant chapters
2. No TOC? Search for "error" → get page numbers
3. Extract those pages + neighbors (~10-20 pages)
4. If answer not found → expand to adjacent sections
5. Never load all 300 pages at once
```

### PDF Decision Tree

```
PDF file received
├─ Small (<20 pages)? → Extract all text
└─ Large (20+ pages)?
   ├─ Has TOC/outline? → Extract by section
   ├─ Specific question? → Search + context pages
   ├─ Need tables? → pdfplumber with table extraction
   └─ Unknown need? → Extract first 5 + last 2 pages, then decide
```

### Token Budget Reference

| PDF Size | Full Extract | Smart Chunking |
|----------|--------------|----------------|
| 50 pages | ~25K tokens | ~25K (just load it) |
| 200 pages | ~100K tokens | ~5-15K (relevant sections) |
| 500 pages | ~250K tokens | ~10-25K (TOC + search) |
| 1000+ pages | Context overflow | ~15-30K (targeted extraction) |

## Preloading Heuristics

Consider preloading first 4-8KB when:
- File is text-based (txt, json, csv, log)
- File is small (<100KB total)
- Workflow typically needs file headers

Skip preloading when:
- File is binary (images, executables)
- Workflow rarely accesses file content
- Multiple large files present

## Anti-Patterns

| Don't | Do Instead |
|-------|------------|
| Load all files "just in case" | Load on-demand as needed |
| Full load for text extraction | Use extract_file |
| Multiple peek calls to reconstruct | Single load_file if >50% needed |
| Ignore file metadata | Use size/type to guide decisions |

## Advanced: Virtual File IDs

For very large extractions, extract_file may return a virtual file ID instead of inline content:

```
extract_file(large_report.pdf)
→ { virtual_id: "extracted_123", size: 50000 }

# Then access the extraction progressively:
peek_file("extracted_123", 0, 10000)  # First 10KB
```

See [references/detailed-examples.md](references/detailed-examples.md) for comprehensive workflow examples.
