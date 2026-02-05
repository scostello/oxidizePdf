//! Test native renderer with a real PDF file
//!
//! Run with: cargo run --example render_real_pdf --features native-render -p oxidize-pdf-editor -- <pdf_path> [page_num]

use oxidize_pdf_editor::render::{NativeBackend, PageRenderer};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let pdf_path = args.get(1).map(|s| s.as_str()).unwrap_or_else(|| {
        eprintln!("Usage: render_real_pdf <pdf_path> [page_num]");
        std::process::exit(1);
    });

    let page_num: usize = args.get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let pdf_path = Path::new(pdf_path);
    println!("Rendering: {} (page {})", pdf_path.display(), page_num);

    // Create native backend
    let backend = NativeBackend::new();
    let scale = 1.5; // 1.5x scale

    // Get page dimensions
    let (width, height) = backend.page_dimensions(pdf_path, page_num)?;
    println!("Page dimensions: {:.0}x{:.0} points", width, height);

    // Render the page
    println!("Rendering with native backend...");
    let image = backend.render_page(pdf_path, page_num, scale)?;
    println!(
        "Rendered image: {}x{} pixels",
        image.width(),
        image.height()
    );

    // Save as PNG
    let output_path = Path::new("target/real_pdf_render.png");
    image.save(output_path)?;
    println!("Saved: {}", output_path.display());

    // Count non-white pixels
    let mut non_white_count = 0;
    for pixel in image.pixels() {
        if pixel[0] != 255 || pixel[1] != 255 || pixel[2] != 255 {
            non_white_count += 1;
        }
    }
    let total_pixels = image.width() * image.height();
    println!(
        "Non-white pixels: {} ({:.2}%)",
        non_white_count,
        100.0 * non_white_count as f64 / total_pixels as f64
    );

    Ok(())
}
