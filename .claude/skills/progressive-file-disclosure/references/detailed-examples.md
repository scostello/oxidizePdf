# Detailed Workflow Examples

## Table of Contents

1. [Email Attachment Processing](#email-attachment-processing)
2. [Data Export Analysis](#data-export-analysis)
3. [Contract Review Workflow](#contract-review-workflow)
4. [Multi-Format Report Generation](#multi-format-report-generation)
5. [Binary File Inspection](#binary-file-inspection)
6. [Large PDF Chunking](#large-pdf-chunking)

---

## Email Attachment Processing

### Scenario
User forwards an email with 8 attachments asking "What's in these files?"

### File Metadata
```
attachments:
  - id: att_001, name: invoice_2024.pdf, size: 245KB, type: application/pdf
  - id: att_002, name: contract_v3.docx, size: 1.2MB, type: application/vnd.openxmlformats-officedocument.wordprocessingml.document
  - id: att_003, name: screenshot.png, size: 890KB, type: image/png
  - id: att_004, name: data_export.csv, size: 15MB, type: text/csv
  - id: att_005, name: notes.txt, size: 2KB, type: text/plain
  - id: att_006, name: presentation.pptx, size: 8MB, type: application/vnd.openxmlformats-officedocument.presentationml.presentation
  - id: att_007, name: debug.log, size: 45MB, type: text/plain
  - id: att_008, name: archive.zip, size: 120MB, type: application/zip
```

### Optimal Approach

```
Step 1: Immediate load (tiny file)
  load_file(att_005)  # notes.txt is 2KB - negligible cost

Step 2: Extract documents (structured content)
  extract_file(att_001)  # invoice PDF → key fields
  extract_file(att_002)  # contract DOCX → text content
  extract_file(att_006)  # presentation → slide text

Step 3: Sample large data
  peek_file(att_004, 0, 8192)  # CSV header + first rows
  peek_file(att_007, -16384, -1)  # Last 16KB of log

Step 4: Describe without loading
  att_003: "890KB screenshot - available if needed"
  att_008: "120MB archive - can list contents if needed"
```

### Response Template
```
Here's what I found in your attachments:

**notes.txt**: [full content since it's tiny]

**invoice_2024.pdf**: Invoice #12345 from Acme Corp, dated Jan 15, 2024
  - Amount: $5,432.00
  - Due: Feb 15, 2024

**contract_v3.docx**: Service Agreement between...
  - Key terms: [extracted summary]

**data_export.csv**: 150,000 rows of sales data
  - Columns: date, product_id, quantity, revenue, region
  - Date range: 2023-01-01 to 2024-01-15

**debug.log**: Recent entries show [error summary from tail]

**presentation.pptx**: 24 slides covering Q4 results...

Would you like me to examine any file in more detail?
```

---

## Data Export Analysis

### Scenario
"Analyze this 50MB JSON export and find all users who signed up last week"

### File Metadata
```
files:
  - id: export_001, name: users_full.json, size: 52MB, type: application/json
```

### Optimal Approach

```
Step 1: Understand structure
  peek_file(export_001, 0, 4096)

  # Reveals:
  {
    "export_date": "2024-01-20",
    "total_records": 125000,
    "schema_version": "2.1",
    "data": [
      {"user_id": "u_001", "email": "...", "created_at": "2024-01-19T..."},
      ...

Step 2: Recognize pattern
  - JSON array with timestamps
  - Need filtering, not full load
  - 52MB would consume entire context

Step 3: Recommend approach
  "This 52MB file contains 125,000 user records. Loading it fully
  would exceed our context. Options:

  A) I can extract and filter server-side if extract_file supports it
  B) You could use jq locally:
     jq '[.data[] | select(.created_at > "2024-01-13")]' users_full.json
  C) I can sample records to understand the data structure better"
```

### Key Insight
Not all files should be loaded. Recognize when to recommend external tooling.

---

## Contract Review Workflow

### Scenario
"Compare these two versions of our vendor agreement and summarize changes"

### File Metadata
```
files:
  - id: contract_v1, name: vendor_agreement_v1.pdf, size: 2.1MB
  - id: contract_v2, name: vendor_agreement_v2.pdf, size: 2.3MB
```

### Optimal Approach

```
Step 1: Extract both documents
  text_v1 = extract_file(contract_v1)  # ~25KB text
  text_v2 = extract_file(contract_v2)  # ~28KB text

Step 2: Compare in context
  - Both extractions fit comfortably
  - Perform section-by-section comparison
  - Identify additions, deletions, modifications

Step 3: Structured output
  ## Contract Comparison Summary

  ### New Sections in V2
  - Section 8.4: Data Retention Requirements
  - Exhibit C: Security Standards

  ### Modified Sections
  - Section 3.2 (Payment Terms): Changed from Net-30 to Net-45
  - Section 5.1 (Liability Cap): Increased from $1M to $2M

  ### Removed Content
  - Section 7.3 (Exclusivity Clause): Deleted entirely
```

---

## Multi-Format Report Generation

### Scenario
"Create a summary report from these quarterly files"

### File Metadata
```
files:
  - id: q1_report, name: Q1_2024_Report.pptx, size: 12MB
  - id: q2_report, name: Q2_2024_Report.pdf, size: 8MB
  - id: q3_report, name: Q3_2024_Report.docx, size: 4MB
  - id: q4_data, name: Q4_2024_Raw.xlsx, size: 25MB
```

### Optimal Approach

```
Step 1: Extract structured documents
  q1_text = extract_file(q1_report)  # PPT → slide text
  q2_text = extract_file(q2_report)  # PDF → document text
  q3_text = extract_file(q3_report)  # DOCX → document text

Step 2: Sample Q4 data
  peek_file(q4_data, 0, 8192)  # See structure/headers

Step 3: Synthesize
  - Q1-Q3: Use extracted text for key metrics
  - Q4: Report data structure, note full analysis needs external tool
```

---

## Binary File Inspection

### Scenario
"What type of file is this? The extension was stripped."

### File Metadata
```
files:
  - id: mystery, name: unknown_file, size: 15MB, type: application/octet-stream
```

### Optimal Approach

```
Step 1: Read magic bytes
  peek_file(mystery, 0, 512)

  # Check for signatures:
  # PDF:  %PDF-
  # PNG:  \x89PNG\r\n\x1a\n
  # ZIP:  PK\x03\x04
  # JPEG: \xFF\xD8\xFF
  # GIF:  GIF87a or GIF89a

Step 2: Report findings
  "Based on the file header (89 50 4E 47), this is a PNG image.
   Size: 15MB suggests a high-resolution image or screenshot.

   Would you like me to:
   A) Extract any embedded text (OCR if available)
   B) Describe the image dimensions/metadata
   C) Just rename it to .png for viewing"
```

---

## Large PDF Chunking

### Scenario
"I have a 400-page technical manual. Help me find how to configure SSL certificates."

### File Metadata
```
files:
  - id: manual_001, name: server_admin_guide.pdf, size: 28MB, pages: 412
```

### Step 1: Get PDF Structure

```python
from pypdf import PdfReader

reader = PdfReader("server_admin_guide.pdf")
print(f"Total pages: {len(reader.pages)}")  # 412

# Check for table of contents
outline = reader.outline
# Returns nested structure:
# [
#   "1. Introduction" (pages 1-15),
#   "2. Installation" (pages 16-45),
#   ...
#   "8. Security Configuration" (pages 201-268),
#     "8.1 Firewall Setup" (pages 201-220),
#     "8.2 SSL/TLS Certificates" (pages 221-245),  # ← TARGET
#     "8.3 Authentication" (pages 246-268),
#   ...
# ]
```

### Step 2: Extract Relevant Section Only

```python
import pdfplumber

def extract_pages(pdf_path, start, end):
    """Extract text from specific page range (1-indexed)."""
    with pdfplumber.open(pdf_path) as pdf:
        texts = []
        for i in range(start - 1, min(end, len(pdf.pages))):
            page_text = pdf.pages[i].extract_text()
            if page_text:
                texts.append(f"--- Page {i + 1} ---\n{page_text}")
        return "\n\n".join(texts)

# Extract only SSL section: pages 221-245 (25 pages)
ssl_content = extract_pages("server_admin_guide.pdf", 221, 245)
# ~12KB of relevant text instead of ~200KB for entire manual
```

### Step 3: If No TOC, Use Search

```python
def search_pdf(pdf_path, keywords, context_pages=2):
    """Find pages containing keywords, return with context."""
    keywords_lower = [k.lower() for k in keywords]
    matches = set()

    with pdfplumber.open(pdf_path) as pdf:
        for i, page in enumerate(pdf.pages):
            text = (page.extract_text() or "").lower()
            if any(kw in text for kw in keywords_lower):
                # Add page and surrounding context
                for j in range(max(0, i - context_pages),
                               min(len(pdf.pages), i + context_pages + 1)):
                    matches.add(j + 1)  # 1-indexed

    return sorted(matches)

# Search for SSL-related content
relevant_pages = search_pdf(
    "server_admin_guide.pdf",
    ["ssl", "certificate", "tls", "https"],
    context_pages=1
)
# Returns: [221, 222, 223, ..., 245, 312, 313, 314]
# Found main section (221-245) + troubleshooting references (312-314)
```

### Optimal Approach Summary

```
Step 1: Metadata (no content loaded)
  - 412 pages, 28MB
  - Has outline/TOC → use it

Step 2: Locate section via TOC
  - "8.2 SSL/TLS Certificates" → pages 221-245

Step 3: Extract targeted pages
  - 25 pages (~12KB) instead of 412 pages (~200KB)
  - 94% token savings

Step 4: If answer incomplete, expand
  - Check related sections (8.1, 8.3)
  - Search for additional references
```

### Response Template
```
I found the SSL certificate configuration in the Server Admin Guide.

**Section 8.2: SSL/TLS Certificates** (pages 221-245)

Key configuration steps:
1. Generate CSR: `openssl req -new -key server.key -out server.csr`
2. Install certificate in /etc/ssl/certs/
3. Update server.conf with certificate paths
4. Restart service: `systemctl restart server`

Common issues covered:
- Certificate chain validation (page 235)
- Mixed content warnings (page 238)
- Certificate renewal automation (page 242)

Would you like me to extract any specific subsection in detail?
```

### Chunking Strategy Comparison

| Approach | Pages Loaded | Tokens | Success Rate |
|----------|--------------|--------|--------------|
| Full extract | 412 | ~200K | Context overflow |
| First 50 pages | 50 | ~25K | Likely miss target |
| TOC-based | 25 | ~12K | High (if TOC exists) |
| Search-based | 28 | ~14K | High (keyword dependent) |
| Hybrid (TOC + search) | 30 | ~15K | Very high |

### Edge Cases

**No TOC/Outline:**
```python
# Fall back to search + sampling
first_pages = extract_pages(pdf, 1, 5)      # Intro/TOC usually here
last_pages = extract_pages(pdf, -3, -1)     # Index/appendix
search_hits = search_pdf(pdf, ["ssl", "certificate"])
```

**Scanned PDF (no text layer):**
```
# pdfplumber returns empty strings
# Options:
# A) Use OCR if available (slow, ~1 page/sec)
# B) Report limitation to user
# C) Check if searchable PDF version exists
```

**Tables and Diagrams:**
```python
# pdfplumber can extract tables
with pdfplumber.open(pdf_path) as pdf:
    page = pdf.pages[230]
    tables = page.extract_tables()
    # Returns list of table data as nested lists
```

---

## Tool Call Cost Reference

| File Size | load_file Cost | extract_file Cost | peek_file(4KB) Cost |
|-----------|----------------|-------------------|---------------------|
| 10KB | ~10KB tokens | ~1-2KB tokens | ~4KB tokens |
| 100KB | ~100KB tokens | ~5-10KB tokens | ~4KB tokens |
| 1MB | ~1MB tokens | ~10-50KB tokens | ~4KB tokens |
| 10MB | ~10MB tokens | ~50-200KB tokens | ~4KB tokens |
| 100MB | Context overflow | ~200KB-1MB tokens | ~4KB tokens |

**Rule of thumb**: extract_file is 10-100x more efficient for documents.
