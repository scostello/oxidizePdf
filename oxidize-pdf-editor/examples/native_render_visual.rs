//! Visual test for native PDF renderer
//!
//! Creates a simple PDF with vector graphics and renders it with the native backend,
//! saving the output as a PNG for visual inspection.
//!
//! Run with: cargo run --example native_render_visual --features native-render -p oxidize-pdf-editor

use oxidize_pdf::{Color, Document, Page};
use oxidize_pdf_editor::render::{NativeBackend, PageRenderer};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a simple PDF with vector graphics
    let mut doc = Document::new();
    doc.set_title("Native Render Test");

    let mut page = Page::a4();

    // Draw some shapes
    let graphics = page.graphics();

    // Red filled rectangle
    graphics
        .set_fill_color(Color::rgb(1.0, 0.0, 0.0))
        .rectangle(100.0, 600.0, 200.0, 100.0)
        .fill();

    // Blue stroked rectangle
    graphics
        .set_stroke_color(Color::rgb(0.0, 0.0, 1.0))
        .set_line_width(3.0)
        .rectangle(350.0, 600.0, 200.0, 100.0)
        .stroke();

    // Green filled circle
    graphics
        .set_fill_color(Color::rgb(0.0, 0.8, 0.0))
        .circle(200.0, 400.0, 50.0)
        .fill();

    // Yellow filled triangle
    graphics
        .set_fill_color(Color::rgb(1.0, 1.0, 0.0))
        .move_to(400.0, 350.0)
        .line_to(500.0, 350.0)
        .line_to(450.0, 450.0)
        .close_path()
        .fill();

    // Gray diagonal line
    graphics
        .set_stroke_color(Color::gray(0.3))
        .set_line_width(5.0)
        .move_to(50.0, 200.0)
        .line_to(550.0, 300.0)
        .stroke();

    doc.add_page(page);

    // Save the PDF
    let pdf_path = Path::new("target/native_render_test.pdf");
    doc.save(pdf_path)?;
    println!("Created test PDF: {}", pdf_path.display());

    // Render with native backend
    let backend = NativeBackend::new();
    let scale = 2.0; // 2x scale for better quality

    // Get page dimensions
    let (width, height) = backend.page_dimensions(pdf_path, 0)?;
    println!("Page dimensions: {}x{} points", width, height);

    // Render the page
    let image = backend.render_page(pdf_path, 0, scale)?;
    println!(
        "Rendered image: {}x{} pixels",
        image.width(),
        image.height()
    );

    // Save as PNG
    let output_path = Path::new("target/native_render_test.png");
    image.save(output_path)?;
    println!("Saved rendered image: {}", output_path.display());

    // Check if rendering produced non-white pixels
    let mut non_white_count = 0;
    for pixel in image.pixels() {
        if pixel[0] != 255 || pixel[1] != 255 || pixel[2] != 255 {
            non_white_count += 1;
        }
    }
    println!(
        "Non-white pixels: {} ({:.2}%)",
        non_white_count,
        100.0 * non_white_count as f64 / (image.width() * image.height()) as f64
    );

    if non_white_count > 0 {
        println!("SUCCESS: Native renderer produced visible content!");
    } else {
        println!("WARNING: Image appears blank - content stream rendering may not be working");
    }

    Ok(())
}
