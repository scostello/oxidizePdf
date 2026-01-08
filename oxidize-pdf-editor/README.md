# oxidize-pdf-editor

A PDF viewer/editor built with iced (UI framework) and oxidize-pdf.

## Current Status: Work in Progress

This is a foundation for a PDF editor with the following features implemented:

✅ **Completed:**
- Project structure with iced + pdfium-render
- Viewport system (zoom, pan state management)
- Tab system for multiple documents
- PDF rendering infrastructure
- Page caching system

⚠️ **Known Issues:**
- pdfium-render types are not Send/Sync, causing threading issues with iced's async runtime
- Need alternative approach for PDF rendering in iced

## Architecture

```
oxidize-pdf-editor/
├── src/
│   ├── main.rs          # Iced application entry point
│   ├── pdf_viewer.rs    # PDF document management & caching
│   ├── renderer.rs      # PDF rendering with pdfium
│   └── viewport.rs      # Zoom/pan state management
└── Cargo.toml
```

## The Problem: pdfium-render + async

The current implementation hits a fundamental limitation:

**pdfium-render is NOT thread-safe:**
- `PdfDocument` contains raw pointers and cannot be sent between threads
- iced's `Task::perform` requires `Send` types for async operations
- This makes it impossible to load/render PDFs asynchronously

**Possible Solutions:**

1. **Use a different PDF rendering library:**
   - `pdf-render` (pure Rust, but less mature)
   - Build custom renderer on top of `oxidize-pdf` parsing + image generation

2. **Blocking I/O approach:**
   - Load PDFs synchronously on button click (blocks UI)
   - Pre-render pages in a separate thread pool
   - Use message passing to send rendered images to UI

3. **Server-client architecture:**
   - PDF rendering server (handles pdfium in single thread)
   - iced client communicates via channels/sockets

## Recommended Next Steps

### Option A: Pure Rust with pdf-render

Replace pdfium-render with pdf-render crate:

```toml
[dependencies]
pdf-render = "0.8"  # Instead of pdfium-render
```

Pros: Thread-safe, pure Rust
Cons: Less mature, may have rendering quality issues

### Option B: Custom Renderer

Leverage oxidize-pdf's excellent PDF parsing with custom rendering:

1. Use `oxidize-pdf` to parse PDFContent streams
2. Manually render graphics operations to image
3. Use `tiny-skia` or `resvg` for 2D graphics

Pros: Full control, leverages oxidize-pdf
Cons: Significant implementation effort

### Option C: Simple Synchronous Viewer

Keep pdfium-render, but load PDFs synchronously:

```rust
// In button handler - blocks UI briefly
fn open_pdf(&mut self, path: PathBuf) {
    match load_pdf_sync(path) {  // Blocking call
        Ok(doc) => self.tabs.push(doc),
        Err(e) => self.show_error(e),
    }
}
```

Pros: Simple, works immediately
Cons: UI freezes during load

## Requirements

**For pdfium-render (current approach):**

Download PDFium library:
- Linux: `libpdfium.so`
- macOS: `libpdfium.dylib`
- Windows: `pdfium.dll`

Get binaries from: https://github.com/bblanchon/pdfium-binaries/releases

Place in project root or system library path.

## Building

```bash
cd oxidize-pdf-editor
cargo build
```

**Expected:** Compilation errors due to threading issues (see above).

## Usage (Future)

Once threading issues are resolved:

```bash
# Run with a PDF file
cargo run -- /path/to/document.pdf

# Or open file from within app
cargo run
```

**Features (planned):**
- Zoom in/out (25% - 400%)
- Pan across page
- Navigate pages
- Multiple tabs
- Annotations (future)
- Form filling (future)

## Contributing

Priority tasks:
1. Resolve pdfium threading issues (see solutions above)
2. Implement file dialog for opening PDFs
3. Add keyboard shortcuts (PgUp/PgDown, +/-, arrow keys)
4. Implement annotation tools

## License

AGPL-3.0 (same as oxidize-pdf)
