# PDF Editor Architecture

## Overview

This PDF editor is built using the Elm Architecture pattern via iced, with a modular design that separates concerns between UI, document management, rendering, and viewport state.

## Component Breakdown

### 1. Main Application (`main.rs`)

**Purpose:** Application entry point and state management

**Key Types:**
```rust
struct PdfEditor {
    tabs: Vec<Tab>,      // Multiple open documents
    active_tab: usize,   // Currently selected tab
}

struct Tab {
    document: PdfDocument,  // PDF document with cache
    viewport: Viewport,      // View state (zoom, pan, page)
}

enum Message {
    OpenFile,
    FileOpened(Result<PdfDocument, String>),
    PageChanged(usize),
    ZoomIn / ZoomOut / ZoomReset,
    Pan(f32, f32),
    CloseTab(usize),
    SelectTab(usize),
}
```

**Responsibilities:**
- Manage application state (tabs, active tab)
- Handle user interactions (messages)
- Coordinate between viewport and rendering
- Render UI (tab bar, toolbar, page viewer)

**Update Flow:**
1. User clicks button → Message generated
2. `update()` processes message → Updates state
3. `view()` renders new UI → Display updated

### 2. PDF Viewer (`pdf_viewer.rs`)

**Purpose:** Document lifecycle and caching

**Key Types:**
```rust
pub struct PdfDocument {
    path: PathBuf,
    document: Document,
    page_cache: HashMap<(usize, u32), Handle>,  // (page, zoom%) → image
}
```

**Responsibilities:**
- Load PDF documents from filesystem
- Cache rendered pages (LRU strategy, max 10 pages)
- Provide metadata (page count, filename)
- Coordinate rendering requests

**Caching Strategy:**
- Key: `(page_index, zoom_percent)`
- Max 10 cached pages
- Simple eviction: remove oldest when limit exceeded
- Opportunity for improvement: proper LRU cache

### 3. Renderer (`renderer.rs`)

**Purpose:** PDF-to-image conversion

**Key Types:**
```rust
pub struct PdfRenderer {
    pdfium: Pdfium,
}

pub struct Document {
    inner: PdfDocument<'static>,  // pdfium document
}
```

**Responsibilities:**
- Initialize PDFium library
- Load PDF documents
- Render pages to RGBA images
- Apply zoom transformations
- Provide page dimensions

**Rendering Process:**
```
PDF Page → pdfium render → Bitmap → RGBA buffer → iced Handle
```

**Configuration:**
- Target width/height based on zoom
- RGBA color format
- No rotation (for now)

### 4. Viewport (`viewport.rs`)

**Purpose:** View state management

**Key Types:**
```rust
pub struct Viewport {
    current_page: usize,
    page_count: usize,
    zoom: f32,           // 0.25 - 4.0 (25% - 400%)
    pan_x: f32,
    pan_y: f32,
}
```

**Responsibilities:**
- Track current page
- Manage zoom level (min 0.25, max 4.0, step 0.25)
- Track pan offset
- Reset pan when changing pages
- Provide navigation methods

**Zoom Levels:**
- Minimum: 25% (0.25x)
- Maximum: 400% (4.0x)
- Step: 25% (0.25x)
- Default: 100% (1.0x)

## Data Flow

### Opening a PDF

```
User clicks "Open" button
    ↓
Message::OpenFile generated
    ↓
Task spawned to load PDF asynchronously  ← THREADING ISSUE HERE
    ↓
PdfDocument::load(path)
    ↓
PdfRenderer::new() → load_document()
    ↓
Message::FileOpened(Result) sent back
    ↓
update() adds to tabs or shows error
    ↓
view() renders new tab
```

### Viewing a Page

```
User navigates to page N
    ↓
Message::PageChanged(N) generated
    ↓
Viewport updates current_page
    ↓
view() requests rendered page
    ↓
PdfDocument::get_rendered_page(N, zoom)
    ↓
Check cache → hit? return cached
    ↓
Cache miss → render page
    ↓
Document::render_page() via pdfium
    ↓
Create iced Handle from RGBA
    ↓
Cache and return Handle
    ↓
Image widget displays Handle
```

### Zooming

```
User clicks "+" button
    ↓
Message::ZoomIn generated
    ↓
Viewport.zoom_in() → zoom += 0.25
    ↓
view() requests page at new zoom
    ↓
Cache miss (different zoom level)
    ↓
Re-render page at new scale
    ↓
Display zoomed page
```

## UI Layout

```
┌─────────────────────────────────────────────────────┐
│ [Tab 1] [Tab 2] [+]                                 │ ← Tab Bar
├─────────────────────────────────────────────────────┤
│ [Open] [−] 100% [+] [Reset]  Page 1/10  [◀] [▶]   │ ← Toolbar
├─────────────────────────────────────────────────────┤
│                                                     │
│              Scrollable Page View                   │
│                                                     │
│                   [PDF Page]                        │
│                                                     │
│                                                     │
└─────────────────────────────────────────────────────┘
```

**Toolbar Controls:**
- Open: Load new PDF
- −/+: Zoom out/in
- Reset: Reset to 100% zoom
- Page counter: Current/total
- ◀/▶: Previous/next page

## Performance Considerations

### Caching
- **Current:** Simple 10-page limit
- **Improvement:** LRU eviction policy
- **Improvement:** Configurable cache size
- **Improvement:** Pre-render adjacent pages

### Rendering
- **Current:** Synchronous, on-demand
- **Improvement:** Background thread pool
- **Improvement:** Progressive rendering for large pages
- **Improvement:** Thumbnail generation

### Memory
- **Current:** Full-page RGBA images in memory
- **Improvement:** Tile-based rendering for large pages
- **Improvement:** Compressed cache storage
- **Improvement:** Memory pressure monitoring

## Threading Issues

### The Problem

pdfium-render types contain raw FFI pointers and are not `Send` + `Sync`:

```rust
// This fails to compile:
Task::perform(
    async {
        // PdfDocument contains non-Send types
        PdfDocument::load(path).await  // ← Error: cannot send between threads
    },
    Message::FileOpened
)
```

### Why It Matters

iced's async runtime expects `Send` futures so tasks can be executed on any thread in the thread pool. pdfium's C library bindings prevent this.

### Solutions

1. **Synchronous Loading:**
   ```rust
   fn open_pdf(&mut self, path: PathBuf) {
       // Blocks UI thread briefly
       let doc = PdfDocument::load_sync(path)?;
       self.tabs.push(doc);
   }
   ```

2. **Dedicated Rendering Thread:**
   ```rust
   // Single thread owns pdfium
   thread::spawn(move || {
       loop {
           let request = rx.recv();
           let image = render_page(request);
           tx.send(image);
       }
   });
   ```

3. **Different Library:**
   - Switch to pure Rust `pdf-render`
   - Build custom renderer on oxidize-pdf

## Extension Points

### Future Features

**Annotations:**
- Add `Annotation` type to `PdfDocument`
- Overlay rendering in `view()`
- Save annotations to PDF with oxidize-pdf

**Editing:**
- Text insertion: Use oxidize-pdf's text API
- Shape drawing: Use graphics context
- Incremental updates: `write_incremental_update()`

**Forms:**
- Leverage oxidize-pdf's AcroForm support
- Field highlighting in viewer
- Interactive field filling

**Search:**
- Text extraction with oxidize-pdf
- Highlight matches in viewer
- Navigation between results

## Technology Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| UI Framework | iced 0.13 | Cross-platform GUI |
| PDF Rendering | pdfium-render 0.8 | Page→Image conversion |
| PDF Manipulation | oxidize-pdf 1.6.7 | Parsing, editing, generation |
| Image Handling | image 0.25 | RGBA buffer management |
| Error Handling | anyhow, thiserror | Error propagation |
| Logging | tracing | Debug logging |

## File Organization

```
oxidize-pdf-editor/
├── src/
│   ├── main.rs           # 217 lines - App structure
│   ├── pdf_viewer.rs     # 98 lines - Document management
│   ├── renderer.rs       # 81 lines - Rendering
│   └── viewport.rs       # 92 lines - View state
├── Cargo.toml            # Dependencies
├── README.md             # User documentation
└── ARCHITECTURE.md       # This file
```

**Total:** ~488 lines of code (excluding docs)

## Next Steps

1. **Resolve threading** - Choose solution from above
2. **File dialog** - Use `rfd` or `native-dialog` crate
3. **Keyboard shortcuts** - Add key event handling
4. **Error UI** - Show error messages to user
5. **Settings** - Theme, default zoom, cache size
6. **Annotations** - Basic drawing tools
7. **Save** - Write modifications back to PDF

## References

- [iced docs](https://docs.rs/iced/)
- [pdfium-render docs](https://docs.rs/pdfium-render/)
- [oxidize-pdf docs](https://docs.rs/oxidize-pdf/)
- [PDFium binaries](https://github.com/bblanchon/pdfium-binaries)
