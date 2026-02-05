//! Font extraction from PDF documents
//!
//! This module extracts embedded font data from PDF pages for use in rendering.
//! It handles the PDF font hierarchy:
//! - Simple fonts: Font → FontDescriptor → FontFile/FontFile2/FontFile3
//! - Type0 fonts: Type0 → DescendantFonts → CIDFont → FontDescriptor → FontFile2/FontFile3

use oxidize_pdf::parser::document::PdfDocument;
use oxidize_pdf::parser::objects::{PdfDictionary, PdfObject, PdfStream};
use oxidize_pdf::parser::page_tree::ParsedPage;
use oxidize_pdf::parser::ParseOptions;
use std::collections::HashMap;
use std::io::{Read, Seek};

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
    // Get FontDescriptor
    let descriptor = get_font_descriptor(font_dict, document)?;

    // Try to extract font data from FontFile, FontFile2, or FontFile3
    let (font_data, font_file_type) = extract_font_file(&descriptor, document);

    Some(ExtractedFont {
        name: font_name.to_string(),
        base_font,
        subtype,
        font_data,
        font_file_type,
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
    let descriptor = get_font_descriptor(&cidfont_dict, document)?;

    // Extract font file
    let (font_data, font_file_type) = extract_font_file(&descriptor, document);

    Some(ExtractedFont {
        name: font_name.to_string(),
        base_font,
        subtype: cidfont_subtype,
        font_data,
        font_file_type,
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
        };

        // Just verify Debug trait works
        let debug_str = format!("{:?}", font);
        assert!(debug_str.contains("F1"));
        assert!(debug_str.contains("Helvetica"));
    }
}
