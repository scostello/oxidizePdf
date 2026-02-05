//! Text rendering with skrifa
//!
//! This module handles rendering PDF text operations using the skrifa font library.
//! It converts glyph outlines to tiny-skia paths for rasterization.
//!
//! ## ToUnicode Support
//!
//! PDF fonts often use custom encodings where bytes in the content stream don't
//! directly correspond to Unicode characters. When a ToUnicode CMap is available,
//! we use it to map character codes to Unicode before looking up glyphs.

use super::font_extractor::ToUnicodeMap;
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::prelude::*;
use skrifa::raw::TableProvider;
use skrifa::MetadataProvider;
use std::collections::HashMap;
use tiny_skia::{Paint, Path, PathBuilder, Pixmap, Transform};

/// Pen implementation that builds tiny-skia paths from glyph outlines
///
/// Note: skrifa's DrawSettings already scales glyph coordinates to the target size,
/// so we only need to apply offset and Y-flip here.
pub struct TinySkiaPen {
    builder: PathBuilder,
    /// Offset for positioning
    offset_x: f32,
    offset_y: f32,
}

impl TinySkiaPen {
    /// Create a new pen with the given offset
    ///
    /// The glyph coordinates from skrifa are already scaled to the target font size,
    /// so we only apply position offset and Y-axis flip.
    pub fn new(offset_x: f32, offset_y: f32) -> Self {
        Self {
            builder: PathBuilder::new(),
            offset_x,
            offset_y,
        }
    }

    /// Consume the pen and return the built path
    pub fn finish(self) -> Option<Path> {
        self.builder.finish()
    }

    /// Transform a point from skrifa coordinates to pixmap coordinates
    ///
    /// skrifa coordinates have Y pointing up, pixmap has Y pointing down.
    fn transform(&self, x: f32, y: f32) -> (f32, f32) {
        (
            x + self.offset_x,
            // Flip Y axis - skrifa has Y up, tiny-skia has Y down
            -y + self.offset_y,
        )
    }
}

impl OutlinePen for TinySkiaPen {
    fn move_to(&mut self, x: f32, y: f32) {
        let (tx, ty) = self.transform(x, y);
        #[cfg(debug_assertions)]
        if self.builder.len() == 0 {
            // Only log the first point
            eprintln!("[TinySkiaPen] First move_to: ({}, {}) -> ({}, {})", x, y, tx, ty);
        }
        self.builder.move_to(tx, ty);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let (tx, ty) = self.transform(x, y);
        self.builder.line_to(tx, ty);
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        let (tcx0, tcy0) = self.transform(cx0, cy0);
        let (tx, ty) = self.transform(x, y);
        self.builder.quad_to(tcx0, tcy0, tx, ty);
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        let (tcx0, tcy0) = self.transform(cx0, cy0);
        let (tcx1, tcy1) = self.transform(cx1, cy1);
        let (tx, ty) = self.transform(x, y);
        self.builder.cubic_to(tcx0, tcy0, tcx1, tcy1, tx, ty);
    }

    fn close(&mut self) {
        self.builder.close();
    }
}

/// Cached font data for rendering
pub struct CachedFont {
    /// Raw font data (kept alive for FontRef)
    #[allow(dead_code)]
    data: Vec<u8>,
    /// Units per em (for scaling)
    #[allow(dead_code)]
    units_per_em: u16,
    /// ToUnicode mapping for character code to Unicode conversion
    to_unicode: Option<ToUnicodeMap>,
    /// Byte width for character codes (from ToUnicode CMap)
    code_byte_width: u8,
}

impl CachedFont {
    /// Create a new cached font from raw data
    pub fn from_data(data: Vec<u8>) -> Option<Self> {
        Self::from_data_with_encoding(data, None)
    }

    /// Create a new cached font with optional ToUnicode mapping
    pub fn from_data_with_encoding(data: Vec<u8>, to_unicode: Option<ToUnicodeMap>) -> Option<Self> {
        // Validate we can parse it
        let font = FontRef::new(&data).ok()?;
        let units_per_em = font.head().ok()?.units_per_em();

        // Extract byte width from ToUnicode map
        let code_byte_width = to_unicode.as_ref().map(|m| m.code_byte_width()).unwrap_or(1);

        #[cfg(debug_assertions)]
        {
            let has_map = to_unicode.as_ref().map(|m| m.len()).unwrap_or(0);
            eprintln!(
                "[CachedFont] Created font with {} upem, {} ToUnicode mappings, code_byte_width={}",
                units_per_em, has_map, code_byte_width
            );
        }

        Some(Self {
            data,
            units_per_em,
            to_unicode,
            code_byte_width,
        })
    }

    /// Get a FontRef for rendering (requires data to be alive)
    pub fn as_font_ref(&self) -> Option<FontRef<'_>> {
        FontRef::new(&self.data).ok()
    }

    /// Get the scale factor for a given font size in points
    #[allow(dead_code)]
    pub fn scale_for_size(&self, size_points: f32) -> f32 {
        size_points / self.units_per_em as f32
    }

    /// Check if this font has a ToUnicode mapping
    pub fn has_to_unicode(&self) -> bool {
        self.to_unicode.is_some()
    }

    /// Get the byte width for character codes
    pub fn code_byte_width(&self) -> u8 {
        self.code_byte_width
    }

    /// Look up a character code in the ToUnicode map
    pub fn to_unicode(&self, code: u32) -> Option<&str> {
        self.to_unicode.as_ref()?.get(code)
    }
}

/// Text renderer using skrifa for glyph rasterization
pub struct TextRenderer {
    /// Font cache: maps font name to cached font data
    font_cache: HashMap<String, CachedFont>,
    /// Fallback font data (embedded or system)
    fallback_font: Option<CachedFont>,
}

impl TextRenderer {
    /// Create a new text renderer
    pub fn new() -> Self {
        let mut renderer = Self {
            font_cache: HashMap::new(),
            fallback_font: None,
        };

        // Try to load a system fallback font
        renderer.load_fallback_font();

        renderer
    }

    /// Load a system fallback font
    fn load_fallback_font(&mut self) {
        // Try common system font paths
        let fallback_paths = [
            // macOS
            "/System/Library/Fonts/Helvetica.ttc",
            "/System/Library/Fonts/Supplemental/Arial.ttf",
            "/Library/Fonts/Arial.ttf",
            // Linux
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            // Windows
            "C:\\Windows\\Fonts\\arial.ttf",
        ];

        for path in fallback_paths {
            if let Ok(data) = std::fs::read(path) {
                if let Some(font) = CachedFont::from_data(data) {
                    #[cfg(debug_assertions)]
                    eprintln!("[TextRenderer] Loaded fallback font from: {}", path);
                    self.fallback_font = Some(font);
                    return;
                }
            }
        }

        #[cfg(debug_assertions)]
        eprintln!("[TextRenderer] WARNING: No fallback font found!");
    }

    /// Register a font with the renderer
    pub fn register_font(&mut self, name: &str, data: Vec<u8>) {
        self.register_font_with_encoding(name, data, None);
    }

    /// Register a font with optional ToUnicode mapping
    pub fn register_font_with_encoding(
        &mut self,
        name: &str,
        data: Vec<u8>,
        to_unicode: Option<ToUnicodeMap>,
    ) {
        if let Some(font) = CachedFont::from_data_with_encoding(data, to_unicode) {
            self.font_cache.insert(name.to_string(), font);
        }
    }

    /// Get a font by name, falling back to the default if not found
    pub fn get_font(&self, name: &str) -> Option<&CachedFont> {
        self.font_cache
            .get(name)
            .or(self.fallback_font.as_ref())
    }

    /// Render text to a pixmap
    ///
    /// # Arguments
    /// * `text` - Raw text bytes (may be encoded)
    /// * `font_name` - Name of the font to use
    /// * `font_size` - Font size in points
    /// * `x`, `y` - Position in user space coordinates
    /// * `transform` - Additional transform to apply
    /// * `paint` - Paint for filling the glyphs
    /// * `pixmap` - Target pixmap
    pub fn render_text(
        &self,
        text: &[u8],
        font_name: &str,
        font_size: f32,
        x: f32,
        y: f32,
        transform: Transform,
        paint: &Paint,
        pixmap: &mut Pixmap,
    ) {
        let Some(cached_font) = self.get_font(font_name) else {
            return;
        };

        let Some(font) = cached_font.as_font_ref() else {
            return;
        };

        let charmap = font.charmap();

        // Get outline glyphs
        let outlines = font.outline_glyphs();

        let mut cursor_x = x;

        // Simple ASCII decoding for now - PDF text encoding is complex
        // In a full implementation, we'd use ToUnicode CMap or font encoding
        for &byte in text {
            let ch = byte as char;

            // Map character to glyph ID
            let Some(glyph_id) = charmap.map(ch) else {
                // Character not in font, skip it
                cursor_x += font_size * 0.6;
                continue;
            };

            // Get the glyph outline
            if let Some(glyph) = outlines.get(glyph_id) {
                // Create pen at current cursor position
                let mut pen = TinySkiaPen::new(cursor_x, y);

                // Create settings for each glyph (DrawSettings doesn't implement Copy)
                let settings = DrawSettings::unhinted(Size::new(font_size), LocationRef::default());

                // Draw the glyph outline
                if glyph.draw(settings, &mut pen).is_ok() {
                    // Build and fill the path
                    if let Some(path) = pen.finish() {
                        pixmap.fill_path(
                            &path,
                            paint,
                            tiny_skia::FillRule::Winding,
                            transform,
                            None,
                        );
                    }
                }
            }

            // Advance cursor (simplified - should use glyph advance width)
            // In a proper implementation, we'd query the font for advance widths
            cursor_x += font_size * 0.6; // Approximate average character width
        }
    }

    /// Render text with proper glyph advances
    pub fn render_text_with_advances(
        &self,
        text: &[u8],
        font_name: &str,
        font_size: f32,
        x: f32,
        y: f32,
        transform: Transform,
        paint: &Paint,
        pixmap: &mut Pixmap,
    ) {
        let Some(cached_font) = self.get_font(font_name) else {
            #[cfg(debug_assertions)]
            eprintln!("[TextRenderer] No font found for: {}", font_name);
            return;
        };

        let Some(font) = cached_font.as_font_ref() else {
            return;
        };

        // Decode text using ToUnicode map if available
        let decoded_text = self.decode_text(text, cached_font);

        #[cfg(debug_assertions)]
        eprintln!(
            "[TextRenderer] render_text_with_advances: font={}, size={}, pos=({}, {}), has_to_unicode={}, decoded={:?}",
            font_name,
            font_size,
            x,
            y,
            cached_font.has_to_unicode(),
            decoded_text
        );

        let charmap = font.charmap();
        let outlines = font.outline_glyphs();

        // Get glyph metrics for advances
        let glyph_metrics = font.glyph_metrics(Size::new(font_size), LocationRef::default());

        let mut cursor_x = x;
        let mut glyphs_drawn = 0;

        for ch in decoded_text.chars() {
            let Some(glyph_id) = charmap.map(ch) else {
                #[cfg(debug_assertions)]
                if glyphs_drawn == 0 {
                    eprintln!("[TextRenderer] No glyph for char: {:?} (U+{:04X})", ch, ch as u32);
                }
                cursor_x += font_size * 0.6;
                continue;
            };

            if let Some(glyph) = outlines.get(glyph_id) {
                let mut pen = TinySkiaPen::new(cursor_x, y);

                // Create settings for each glyph (DrawSettings doesn't implement Copy)
                let settings = DrawSettings::unhinted(Size::new(font_size), LocationRef::default());

                if glyph.draw(settings, &mut pen).is_ok() {
                    if let Some(path) = pen.finish() {
                        #[cfg(debug_assertions)]
                        if glyphs_drawn == 0 {
                            // Log bounds of first glyph path
                            let bounds = path.bounds();
                            eprintln!(
                                "[TextRenderer] First glyph path bounds: ({}, {}) to ({}, {})",
                                bounds.x(), bounds.y(),
                                bounds.x() + bounds.width(), bounds.y() + bounds.height()
                            );
                        }
                        pixmap.fill_path(
                            &path,
                            paint,
                            tiny_skia::FillRule::Winding,
                            transform,
                            None,
                        );
                        glyphs_drawn += 1;
                    }
                }
            }

            // Use actual glyph advance width
            let advance = glyph_metrics.advance_width(glyph_id).unwrap_or(font_size * 0.6);
            cursor_x += advance;
        }

        #[cfg(debug_assertions)]
        eprintln!("[TextRenderer] Drew {} glyphs", glyphs_drawn);
    }

    /// Decode text bytes using ToUnicode map if available
    ///
    /// If the font has a ToUnicode map, each byte (or multi-byte sequence)
    /// is looked up to get the corresponding Unicode string. Otherwise,
    /// bytes are interpreted as ASCII.
    fn decode_text(&self, text: &[u8], cached_font: &CachedFont) -> String {
        if !cached_font.has_to_unicode() {
            // No ToUnicode map - treat as ASCII
            return text.iter().map(|&b| b as char).collect();
        }

        let byte_width = cached_font.code_byte_width();

        #[cfg(debug_assertions)]
        {
            let hex: Vec<String> = text.iter().map(|b| format!("{:02X}", b)).collect();
            eprintln!(
                "[decode_text] Raw bytes: [{}], byte_width={}",
                hex.join(" "),
                byte_width
            );
        }

        let mut result = String::new();
        let mut i = 0;

        while i < text.len() {
            if byte_width == 2 && i + 1 < text.len() {
                // 2-byte codes: combine two bytes into one code
                let code = ((text[i] as u32) << 8) | (text[i + 1] as u32);
                if let Some(unicode) = cached_font.to_unicode(code) {
                    #[cfg(debug_assertions)]
                    if result.is_empty() {
                        eprintln!("[decode_text] 2-byte code 0x{:04X} -> {:?}", code, unicode);
                    }
                    result.push_str(unicode);
                } else {
                    #[cfg(debug_assertions)]
                    if result.is_empty() {
                        eprintln!("[decode_text] No mapping for 2-byte code 0x{:04X}", code);
                    }
                }
                i += 2;
            } else {
                // 1-byte codes
                let code = text[i] as u32;
                if let Some(unicode) = cached_font.to_unicode(code) {
                    #[cfg(debug_assertions)]
                    if result.is_empty() {
                        eprintln!("[decode_text] 1-byte code 0x{:02X} -> {:?}", code, unicode);
                    }
                    result.push_str(unicode);
                } else {
                    // Fall back to treating the byte as a character
                    #[cfg(debug_assertions)]
                    if result.is_empty() {
                        eprintln!("[decode_text] No mapping for 1-byte code 0x{:02X}", code);
                    }
                    // Don't add null or control characters
                    if text[i] >= 0x20 {
                        result.push(text[i] as char);
                    }
                }
                i += 1;
            }
        }

        result
    }
}

impl Default for TextRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tiny_skia_pen_transform() {
        let pen = TinySkiaPen::new(100.0, 200.0);
        let (x, y) = pen.transform(10.0, 5.0);

        // x = 10 + 100 = 110
        assert!((x - 110.0).abs() < 0.01);
        // y = -5 + 200 = 195 (flipped)
        assert!((y - 195.0).abs() < 0.01);
    }

    #[test]
    fn test_text_renderer_creation() {
        let renderer = TextRenderer::new();
        // Just test that it creates without panicking
        assert!(renderer.font_cache.is_empty());
    }

    #[test]
    fn test_register_font() {
        let mut renderer = TextRenderer::new();

        // Register a minimal valid TTF (this won't actually work without real font data)
        // In real tests, we'd use actual font files
        let fake_font_data = vec![0u8; 100];
        renderer.register_font("TestFont", fake_font_data);

        // Font won't be registered because data is invalid
        assert!(renderer.font_cache.is_empty());
    }
}
