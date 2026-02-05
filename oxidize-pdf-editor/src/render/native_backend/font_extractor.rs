//! Font extraction from PDF documents
//!
//! This module extracts embedded font data from PDF pages for use in rendering.
//! It handles the PDF font hierarchy:
//! - Simple fonts: Font → FontDescriptor → FontFile/FontFile2/FontFile3
//! - Type0 fonts: Type0 → DescendantFonts → CIDFont → FontDescriptor → FontFile2/FontFile3
//!
//! ## ToUnicode CMap Support
//!
//! PDF fonts often use custom encodings where byte values in content streams don't
//! correspond directly to Unicode characters. The ToUnicode CMap provides the mapping
//! from character codes (bytes in the PDF) to Unicode code points.
//!
//! This module parses ToUnicode CMaps and provides the mapping to the text renderer.

use oxidize_pdf::parser::document::PdfDocument;
use oxidize_pdf::parser::objects::{PdfDictionary, PdfObject, PdfStream};
use oxidize_pdf::parser::page_tree::ParsedPage;
use oxidize_pdf::parser::ParseOptions;
use std::collections::HashMap;
use std::io::{Read, Seek};

/// Mapping from character codes to Unicode strings
///
/// PDF ToUnicode CMaps can map single bytes or multi-byte sequences to
/// Unicode strings (potentially multiple code points).
#[derive(Debug, Clone, Default)]
pub struct ToUnicodeMap {
    /// Maps character codes (as u32 for multi-byte support) to Unicode strings
    mappings: HashMap<u32, String>,
    /// The byte width of character codes in this CMap (1 or 2)
    /// This is determined by the hex string length in the CMap
    code_byte_width: u8,
}

impl ToUnicodeMap {
    /// Create a new empty mapping
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
            code_byte_width: 1, // Default to 1-byte codes
        }
    }

    /// Add a mapping from a character code to a Unicode string
    pub fn insert(&mut self, code: u32, unicode: String) {
        self.mappings.insert(code, unicode);
    }

    /// Set the byte width for character codes
    pub fn set_code_byte_width(&mut self, width: u8) {
        self.code_byte_width = width;
    }

    /// Get the byte width for character codes (1 or 2)
    pub fn code_byte_width(&self) -> u8 {
        self.code_byte_width
    }

    /// Look up a character code and return the Unicode string
    pub fn get(&self, code: u32) -> Option<&str> {
        self.mappings.get(&code).map(|s| s.as_str())
    }

    /// Check if the map has any entries
    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }

    /// Number of mappings
    pub fn len(&self) -> usize {
        self.mappings.len()
    }
}

/// Extracted font information
#[derive(Debug)]
pub struct ExtractedFont {
    /// Font name (as used in PDF content streams, e.g., "F1", "TT0")
    pub name: String,
    /// Base font name (PostScript name, e.g., "Helvetica", "ArialMT")
    pub base_font: Option<String>,
    /// Font subtype (Type1, TrueType, Type0, CIDFontType2, etc.)
    pub subtype: Option<String>,
    /// Decoded font program data (if embedded)
    pub font_data: Option<Vec<u8>>,
    /// Font file type (for determining how to parse the data)
    pub font_file_type: FontFileType,
    /// ToUnicode CMap for character code to Unicode mapping
    pub to_unicode: Option<ToUnicodeMap>,
}

/// Type of font file embedded in the PDF
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontFileType {
    /// Type 1 font program (FontFile)
    Type1,
    /// TrueType font (FontFile2)
    TrueType,
    /// CFF/OpenType font (FontFile3)
    CFF,
    /// No embedded font data
    None,
}

/// Extract fonts from a parsed PDF page
///
/// Returns a map of font name → extracted font info with embedded data (if available)
pub fn extract_page_fonts<R: Read + Seek>(
    page: &ParsedPage,
    document: &PdfDocument<R>,
) -> HashMap<String, ExtractedFont> {
    let mut fonts = HashMap::new();

    // Get page resources
    let Some(resources) = page.get_resources() else {
        return fonts;
    };

    // Get Font dictionary from resources
    let Some(font_dict) = resources.get("Font").and_then(|f| f.as_dict()) else {
        return fonts;
    };

    // Iterate through fonts
    for (font_name, font_obj) in &font_dict.0 {
        let name_str = font_name.as_str();
        if let Some(extracted) = extract_font(name_str, font_obj, document) {
            fonts.insert(name_str.to_string(), extracted);
        }
    }

    fonts
}

/// Extract a single font from its dictionary or reference
fn extract_font<R: Read + Seek>(
    font_name: &str,
    font_obj: &PdfObject,
    document: &PdfDocument<R>,
) -> Option<ExtractedFont> {
    // Resolve font dictionary if it's a reference
    let font_dict = match font_obj {
        PdfObject::Dictionary(dict) => dict.clone(),
        PdfObject::Reference(num, generation) => {
            let resolved = document.get_object(*num, *generation).ok()?;
            match resolved {
                PdfObject::Dictionary(dict) => dict.clone(),
                _ => return None,
            }
        }
        _ => return None,
    };

    // Get font subtype
    let subtype = font_dict
        .get("Subtype")
        .and_then(|s| s.as_name())
        .map(|n| n.0.clone());

    // Get base font name
    let base_font = font_dict
        .get("BaseFont")
        .and_then(|b| b.as_name())
        .map(|n| n.0.clone());

    // Check if this is a Type0 (composite) font
    let is_type0 = subtype.as_deref() == Some("Type0");

    if is_type0 {
        extract_type0_font(font_name, &font_dict, base_font, document)
    } else {
        extract_simple_font(font_name, &font_dict, subtype, base_font, document)
    }
}

/// Extract a simple font (Type1, TrueType, etc.)
fn extract_simple_font<R: Read + Seek>(
    font_name: &str,
    font_dict: &PdfDictionary,
    subtype: Option<String>,
    base_font: Option<String>,
    document: &PdfDocument<R>,
) -> Option<ExtractedFont> {
    // Get FontDescriptor (may not exist for standard fonts)
    let descriptor = get_font_descriptor(font_dict, document);

    // Try to extract font data from FontFile, FontFile2, or FontFile3
    let (font_data, font_file_type) = if let Some(ref desc) = descriptor {
        extract_font_file(desc, document)
    } else {
        (None, FontFileType::None)
    };

    // Extract ToUnicode CMap
    let to_unicode = extract_to_unicode(font_dict, document);

    Some(ExtractedFont {
        name: font_name.to_string(),
        base_font,
        subtype,
        font_data,
        font_file_type,
        to_unicode,
    })
}

/// Extract a Type0 (composite) font
fn extract_type0_font<R: Read + Seek>(
    font_name: &str,
    font_dict: &PdfDictionary,
    base_font: Option<String>,
    document: &PdfDocument<R>,
) -> Option<ExtractedFont> {
    // Get DescendantFonts array
    let descendants = font_dict.get("DescendantFonts")?;

    // Resolve if reference
    let descendants_array = match descendants {
        PdfObject::Array(arr) => arr.clone(),
        PdfObject::Reference(num, generation) => {
            let resolved = document.get_object(*num, *generation).ok()?;
            match resolved {
                PdfObject::Array(arr) => arr.clone(),
                _ => return None,
            }
        }
        _ => return None,
    };

    // Get first descendant (CIDFont)
    let cidfont_obj = descendants_array.0.first()?;

    let cidfont_dict = match cidfont_obj {
        PdfObject::Dictionary(dict) => dict.clone(),
        PdfObject::Reference(num, generation) => {
            let resolved = document.get_object(*num, *generation).ok()?;
            match resolved {
                PdfObject::Dictionary(dict) => dict.clone(),
                _ => return None,
            }
        }
        _ => return None,
    };

    // Get CIDFont subtype
    let cidfont_subtype = cidfont_dict
        .get("Subtype")
        .and_then(|s| s.as_name())
        .map(|n| n.0.clone());

    // Get FontDescriptor from CIDFont
    let descriptor = get_font_descriptor(&cidfont_dict, document);

    // Extract font file
    let (font_data, font_file_type) = if let Some(ref desc) = descriptor {
        extract_font_file(desc, document)
    } else {
        (None, FontFileType::None)
    };

    // Extract ToUnicode CMap (from the Type0 font dict, not the CIDFont)
    let to_unicode = extract_to_unicode(font_dict, document);

    Some(ExtractedFont {
        name: font_name.to_string(),
        base_font,
        subtype: cidfont_subtype,
        font_data,
        font_file_type,
        to_unicode,
    })
}

/// Get FontDescriptor dictionary from a font dictionary
fn get_font_descriptor<R: Read + Seek>(
    font_dict: &PdfDictionary,
    document: &PdfDocument<R>,
) -> Option<PdfDictionary> {
    let descriptor_obj = font_dict.get("FontDescriptor")?;

    match descriptor_obj {
        PdfObject::Dictionary(dict) => Some(dict.clone()),
        PdfObject::Reference(num, generation) => {
            let resolved = document.get_object(*num, *generation).ok()?;
            match resolved {
                PdfObject::Dictionary(dict) => Some(dict.clone()),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Extract font file data from FontDescriptor
///
/// Tries FontFile, FontFile2, and FontFile3 in that order
fn extract_font_file<R: Read + Seek>(
    descriptor: &PdfDictionary,
    document: &PdfDocument<R>,
) -> (Option<Vec<u8>>, FontFileType) {
    // Try FontFile (Type 1)
    if let Some(data) = try_extract_stream(descriptor.get("FontFile"), document) {
        return (Some(data), FontFileType::Type1);
    }

    // Try FontFile2 (TrueType)
    if let Some(data) = try_extract_stream(descriptor.get("FontFile2"), document) {
        return (Some(data), FontFileType::TrueType);
    }

    // Try FontFile3 (CFF/OpenType)
    if let Some(data) = try_extract_stream(descriptor.get("FontFile3"), document) {
        return (Some(data), FontFileType::CFF);
    }

    (None, FontFileType::None)
}

/// Try to extract and decode a stream object
fn try_extract_stream<R: Read + Seek>(
    obj: Option<&PdfObject>,
    document: &PdfDocument<R>,
) -> Option<Vec<u8>> {
    let obj = obj?;

    let stream = match obj {
        PdfObject::Stream(stream) => stream.clone(),
        PdfObject::Reference(num, generation) => {
            let resolved = document.get_object(*num, *generation).ok()?;
            match resolved {
                PdfObject::Stream(stream) => stream.clone(),
                _ => return None,
            }
        }
        _ => return None,
    };

    // Decode the stream
    decode_font_stream(&stream)
}

/// Decode a font stream, handling compression
fn decode_font_stream(stream: &PdfStream) -> Option<Vec<u8>> {
    // Try to decode with filters
    let options = ParseOptions::default();
    match stream.decode(&options) {
        Ok(data) => {
            if !data.is_empty() {
                Some(data)
            } else {
                // Fall back to raw data if decode returns empty
                let raw = stream.raw_data();
                if !raw.is_empty() {
                    Some(raw.to_vec())
                } else {
                    None
                }
            }
        }
        Err(_) => {
            // Decode failed, try raw data
            let raw = stream.raw_data();
            if !raw.is_empty() {
                Some(raw.to_vec())
            } else {
                None
            }
        }
    }
}

/// Extract ToUnicode CMap from a font dictionary
fn extract_to_unicode<R: Read + Seek>(
    font_dict: &PdfDictionary,
    document: &PdfDocument<R>,
) -> Option<ToUnicodeMap> {
    let to_unicode_obj = font_dict.get("ToUnicode")?;

    // Get the stream data
    let stream_data = match to_unicode_obj {
        PdfObject::Stream(stream) => decode_font_stream(stream)?,
        PdfObject::Reference(num, generation) => {
            let resolved = document.get_object(*num, *generation).ok()?;
            match resolved {
                PdfObject::Stream(stream) => decode_font_stream(&stream)?,
                _ => return None,
            }
        }
        _ => return None,
    };

    // Parse the CMap
    parse_to_unicode_cmap(&stream_data)
}

/// Parse a ToUnicode CMap stream into a mapping
///
/// ToUnicode CMaps have a specific format with operators like:
/// - `beginbfchar` / `endbfchar` for single character mappings
/// - `beginbfrange` / `endbfrange` for range mappings
///
/// Example:
/// ```text
/// 1 beginbfchar
/// <0003> <0048>
/// endbfchar
/// 2 beginbfrange
/// <0004> <0005> <0065>
/// <0010> <0012> [<0041> <0042> <0043>]
/// endbfrange
/// ```
fn parse_to_unicode_cmap(data: &[u8]) -> Option<ToUnicodeMap> {
    let text = String::from_utf8_lossy(data);
    let mut map = ToUnicodeMap::new();

    // Detect byte width from codespacerange (if present)
    // Format: <00> <FF> for 1-byte, <0000> <FFFF> for 2-byte
    if let Some(width) = detect_code_byte_width(&text) {
        map.set_code_byte_width(width);
    }

    // Parse bfchar sections (single character mappings)
    parse_bfchar_sections(&text, &mut map);

    // Parse bfrange sections (range mappings)
    parse_bfrange_sections(&text, &mut map);

    if map.is_empty() {
        None
    } else {
        #[cfg(debug_assertions)]
        eprintln!(
            "[ToUnicode] Parsed {} mappings, code_byte_width={}",
            map.len(),
            map.code_byte_width()
        );
        Some(map)
    }
}

/// Detect the byte width of character codes from the codespacerange
fn detect_code_byte_width(text: &str) -> Option<u8> {
    // Look for begincodespacerange section
    // Format: 1 begincodespacerange <00> <FF> endcodespacerange (1-byte)
    //     or: 1 begincodespacerange <0000> <FFFF> endcodespacerange (2-byte)
    if let Some(start) = text.find("begincodespacerange") {
        if let Some(end) = text[start..].find("endcodespacerange") {
            let section = &text[start..start + end];
            // Find the first hex code to determine width
            if let Some(hex_start) = section.find('<') {
                if let Some(hex_end) = section[hex_start + 1..].find('>') {
                    let hex_len = hex_end;
                    // 2 hex chars = 1 byte, 4 hex chars = 2 bytes
                    let byte_width = (hex_len / 2) as u8;
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "[ToUnicode] Detected code byte width: {} (from codespacerange)",
                        byte_width
                    );
                    return Some(byte_width);
                }
            }
        }
    }

    // Fallback: look at the first bfchar or bfrange entry
    if let Some(start) = text.find("beginbfchar") {
        let section_start = start + "beginbfchar".len();
        if let Some(hex_start) = text[section_start..].find('<') {
            if let Some(hex_end) = text[section_start + hex_start + 1..].find('>') {
                let hex_len = hex_end;
                let byte_width = (hex_len / 2).max(1) as u8;
                #[cfg(debug_assertions)]
                eprintln!(
                    "[ToUnicode] Detected code byte width: {} (from bfchar)",
                    byte_width
                );
                return Some(byte_width);
            }
        }
    }

    None
}

/// Parse `beginbfchar` / `endbfchar` sections
fn parse_bfchar_sections(text: &str, map: &mut ToUnicodeMap) {
    let mut remaining = text;

    while let Some(start_idx) = remaining.find("beginbfchar") {
        // Find the matching endbfchar
        let section_start = start_idx + "beginbfchar".len();
        let Some(end_idx) = remaining[section_start..].find("endbfchar") else {
            break;
        };

        let section = &remaining[section_start..section_start + end_idx];

        // Parse each line: <srcCode> <dstString>
        for line in section.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Extract hex strings
            let parts: Vec<&str> = line
                .split('<')
                .filter_map(|s| s.split('>').next())
                .filter(|s| !s.is_empty())
                .collect();

            if parts.len() >= 2 {
                if let (Some(src), Some(dst)) = (
                    parse_hex_code(parts[0]),
                    parse_hex_to_unicode(parts[1]),
                ) {
                    map.insert(src, dst);
                }
            }
        }

        remaining = &remaining[section_start + end_idx + "endbfchar".len()..];
    }
}

/// Parse `beginbfrange` / `endbfrange` sections
fn parse_bfrange_sections(text: &str, map: &mut ToUnicodeMap) {
    let mut remaining = text;

    while let Some(start_idx) = remaining.find("beginbfrange") {
        let section_start = start_idx + "beginbfrange".len();
        let Some(end_idx) = remaining[section_start..].find("endbfrange") else {
            break;
        };

        let section = &remaining[section_start..section_start + end_idx];

        // Parse each line: <srcCodeLo> <srcCodeHi> <dstStringLo>
        // Or: <srcCodeLo> <srcCodeHi> [<dst1> <dst2> ...]
        for line in section.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Check if this is an array mapping
            if line.contains('[') {
                parse_bfrange_array(line, map);
            } else {
                parse_bfrange_simple(line, map);
            }
        }

        remaining = &remaining[section_start + end_idx + "endbfrange".len()..];
    }
}

/// Parse a simple bfrange line: <srcLo> <srcHi> <dstLo>
fn parse_bfrange_simple(line: &str, map: &mut ToUnicodeMap) {
    let parts: Vec<&str> = line
        .split('<')
        .filter_map(|s| s.split('>').next())
        .filter(|s| !s.is_empty())
        .collect();

    if parts.len() >= 3 {
        if let (Some(src_lo), Some(src_hi), Some(dst_lo)) = (
            parse_hex_code(parts[0]),
            parse_hex_code(parts[1]),
            parse_hex_code(parts[2]),
        ) {
            // Map each code in the range
            for (i, src) in (src_lo..=src_hi).enumerate() {
                let dst = dst_lo + i as u32;
                if let Some(ch) = char::from_u32(dst) {
                    map.insert(src, ch.to_string());
                }
            }
        }
    }
}

/// Parse a bfrange array line: <srcLo> <srcHi> [<dst1> <dst2> ...]
fn parse_bfrange_array(line: &str, map: &mut ToUnicodeMap) {
    // Extract the hex codes before the array
    let bracket_idx = line.find('[').unwrap_or(line.len());
    let prefix = &line[..bracket_idx];

    let parts: Vec<&str> = prefix
        .split('<')
        .filter_map(|s| s.split('>').next())
        .filter(|s| !s.is_empty())
        .collect();

    if parts.len() < 2 {
        return;
    }

    let Some(src_lo) = parse_hex_code(parts[0]) else {
        return;
    };
    let Some(src_hi) = parse_hex_code(parts[1]) else {
        return;
    };

    // Extract array elements
    let Some(array_start) = line.find('[') else {
        return;
    };
    let Some(array_end) = line.find(']') else {
        return;
    };
    let array_content = &line[array_start + 1..array_end];

    let dst_codes: Vec<String> = array_content
        .split('<')
        .filter_map(|s| s.split('>').next())
        .filter(|s| !s.is_empty())
        .filter_map(|s| parse_hex_to_unicode(s))
        .collect();

    // Map each source code to its corresponding destination
    for (i, src) in (src_lo..=src_hi).enumerate() {
        if i < dst_codes.len() {
            map.insert(src, dst_codes[i].clone());
        }
    }
}

/// Parse a hex string as a character code (1-4 bytes)
fn parse_hex_code(hex: &str) -> Option<u32> {
    u32::from_str_radix(hex.trim(), 16).ok()
}

/// Parse a hex string as a Unicode string
///
/// The hex represents UTF-16BE encoded data.
/// E.g., "0048" -> "H", "00480065006C006C006F" -> "Hello"
fn parse_hex_to_unicode(hex: &str) -> Option<String> {
    let hex = hex.trim();
    if hex.is_empty() {
        return None;
    }

    // Parse as UTF-16BE
    let bytes: Vec<u8> = (0..hex.len())
        .step_by(2)
        .filter_map(|i| {
            if i + 2 <= hex.len() {
                u8::from_str_radix(&hex[i..i + 2], 16).ok()
            } else {
                None
            }
        })
        .collect();

    // Convert pairs of bytes to u16 (big-endian)
    let code_units: Vec<u16> = bytes
        .chunks(2)
        .filter_map(|chunk| {
            if chunk.len() == 2 {
                Some(u16::from_be_bytes([chunk[0], chunk[1]]))
            } else if chunk.len() == 1 {
                // Single byte - treat as direct code point
                Some(chunk[0] as u16)
            } else {
                None
            }
        })
        .collect();

    // Decode UTF-16
    String::from_utf16(&code_units).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_file_type_variants() {
        assert_ne!(FontFileType::Type1, FontFileType::TrueType);
        assert_ne!(FontFileType::TrueType, FontFileType::CFF);
        assert_eq!(FontFileType::None, FontFileType::None);
    }

    #[test]
    fn test_extracted_font_debug() {
        let font = ExtractedFont {
            name: "F1".to_string(),
            base_font: Some("Helvetica".to_string()),
            subtype: Some("TrueType".to_string()),
            font_data: None,
            font_file_type: FontFileType::None,
            to_unicode: None,
        };

        // Just verify Debug trait works
        let debug_str = format!("{:?}", font);
        assert!(debug_str.contains("F1"));
        assert!(debug_str.contains("Helvetica"));
    }

    #[test]
    fn test_to_unicode_map() {
        let mut map = ToUnicodeMap::new();
        assert!(map.is_empty());

        map.insert(0x0003, "H".to_string());
        map.insert(0x0004, "e".to_string());
        map.insert(0x0005, "l".to_string());

        assert_eq!(map.len(), 3);
        assert_eq!(map.get(0x0003), Some("H"));
        assert_eq!(map.get(0x0004), Some("e"));
        assert_eq!(map.get(0x0005), Some("l"));
        assert_eq!(map.get(0x0006), None);
    }

    #[test]
    fn test_parse_hex_code() {
        assert_eq!(parse_hex_code("0003"), Some(0x0003));
        assert_eq!(parse_hex_code("00FF"), Some(0x00FF));
        assert_eq!(parse_hex_code("1234"), Some(0x1234));
        assert_eq!(parse_hex_code("FFFF"), Some(0xFFFF));
    }

    #[test]
    fn test_parse_hex_to_unicode() {
        // Single character: "H" is U+0048
        assert_eq!(parse_hex_to_unicode("0048"), Some("H".to_string()));

        // Multiple characters: "He" is U+0048 U+0065
        assert_eq!(parse_hex_to_unicode("00480065"), Some("He".to_string()));

        // Empty
        assert_eq!(parse_hex_to_unicode(""), None);
    }

    #[test]
    fn test_parse_bfchar() {
        let cmap = r#"
/CIDInit /ProcSet findresource begin
1 beginbfchar
<0003> <0048>
endbfchar
endcmap
"#;
        let map = parse_to_unicode_cmap(cmap.as_bytes()).unwrap();
        assert_eq!(map.get(0x0003), Some("H"));
    }

    #[test]
    fn test_parse_bfrange_simple() {
        let cmap = r#"
1 beginbfrange
<0041> <0043> <0061>
endbfrange
"#;
        let map = parse_to_unicode_cmap(cmap.as_bytes()).unwrap();
        // 0x0041 -> 0x0061 ('a')
        // 0x0042 -> 0x0062 ('b')
        // 0x0043 -> 0x0063 ('c')
        assert_eq!(map.get(0x0041), Some("a"));
        assert_eq!(map.get(0x0042), Some("b"));
        assert_eq!(map.get(0x0043), Some("c"));
    }

    #[test]
    fn test_parse_bfrange_array() {
        let cmap = r#"
1 beginbfrange
<0001> <0003> [<0048> <0065> <006C>]
endbfrange
"#;
        let map = parse_to_unicode_cmap(cmap.as_bytes()).unwrap();
        // 0x0001 -> "H"
        // 0x0002 -> "e"
        // 0x0003 -> "l"
        assert_eq!(map.get(0x0001), Some("H"));
        assert_eq!(map.get(0x0002), Some("e"));
        assert_eq!(map.get(0x0003), Some("l"));
    }

    #[test]
    fn test_parse_multiple_sections() {
        let cmap = r#"
2 beginbfchar
<0001> <0048>
<0002> <0065>
endbfchar
1 beginbfrange
<0010> <0012> <0041>
endbfrange
"#;
        let map = parse_to_unicode_cmap(cmap.as_bytes()).unwrap();
        assert_eq!(map.get(0x0001), Some("H"));
        assert_eq!(map.get(0x0002), Some("e"));
        assert_eq!(map.get(0x0010), Some("A"));
        assert_eq!(map.get(0x0011), Some("B"));
        assert_eq!(map.get(0x0012), Some("C"));
    }
}
