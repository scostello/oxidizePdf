//! Native Rust PDF rendering backend
//!
//! This module implements PDF page rendering using pure Rust libraries:
//! - `tiny-skia` for 2D rasterization
//! - `skrifa` for font parsing and glyph outlines
//!
//! # Architecture
//!
//! The renderer processes PDF content streams by:
//! 1. Parsing operators via oxidize-pdf's ContentParser
//! 2. Maintaining graphics state (CTM, colors, line properties)
//! 3. Converting PDF paths to tiny-skia paths
//! 4. Rendering text using skrifa for glyph outlines
//! 5. Rasterizing to an RGBA bitmap

mod font_extractor;
mod graphics_state;
mod path_builder;
mod renderer;
mod text_renderer;

pub use renderer::NativeBackend;

use super::{PageRenderer, RenderError};
