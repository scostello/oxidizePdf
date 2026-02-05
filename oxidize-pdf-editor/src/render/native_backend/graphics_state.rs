//! PDF Graphics State
//!
//! Tracks the current graphics state during PDF rendering, including:
//! - Current transformation matrix (CTM)
//! - Fill and stroke colors
//! - Line properties (width, cap, join, dash)
//! - Clipping path
//! - Text state

use tiny_skia::{Color, LineCap, LineJoin, Transform};

/// Line dash pattern
#[derive(Debug, Clone)]
pub struct DashPattern {
    pub array: Vec<f32>,
    pub phase: f32,
}

/// Text rendering state
#[derive(Debug, Clone)]
pub struct TextState {
    /// Character spacing (Tc)
    pub char_spacing: f32,
    /// Word spacing (Tw)
    pub word_spacing: f32,
    /// Horizontal scaling (Tz), as percentage (100 = normal)
    pub horizontal_scaling: f32,
    /// Leading (TL)
    pub leading: f32,
    /// Current font name
    pub font_name: Option<String>,
    /// Current font size
    pub font_size: f32,
    /// Text rendering mode (Tr)
    pub render_mode: i32,
    /// Text rise (Ts)
    pub rise: f32,
    /// Text matrix (Tm)
    pub text_matrix: Transform,
    /// Text line matrix
    pub text_line_matrix: Transform,
}

impl Default for TextState {
    fn default() -> Self {
        Self {
            char_spacing: 0.0,
            word_spacing: 0.0,
            horizontal_scaling: 100.0,
            leading: 0.0,
            font_name: None,
            font_size: 12.0,
            render_mode: 0, // Fill
            rise: 0.0,
            text_matrix: Transform::identity(),
            text_line_matrix: Transform::identity(),
        }
    }
}

/// Complete PDF graphics state
#[derive(Debug, Clone)]
pub struct GraphicsState {
    /// Current transformation matrix
    pub ctm: Transform,

    /// Fill color
    pub fill_color: Color,

    /// Stroke color
    pub stroke_color: Color,

    /// Line width
    pub line_width: f32,

    /// Line cap style
    pub line_cap: LineCap,

    /// Line join style
    pub line_join: LineJoin,

    /// Miter limit
    pub miter_limit: f32,

    /// Dash pattern (None = solid line)
    pub dash_pattern: Option<DashPattern>,

    /// Fill opacity (0.0 - 1.0)
    pub fill_alpha: f32,

    /// Stroke opacity (0.0 - 1.0)
    pub stroke_alpha: f32,

    /// Text state
    pub text: TextState,
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            ctm: Transform::identity(),
            fill_color: Color::BLACK,
            stroke_color: Color::BLACK,
            line_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            miter_limit: 10.0,
            dash_pattern: None,
            fill_alpha: 1.0,
            stroke_alpha: 1.0,
            text: TextState::default(),
        }
    }
}

impl GraphicsState {
    /// Create a new default graphics state
    pub fn new() -> Self {
        Self::default()
    }

    /// Set fill color from RGB values (0.0 - 1.0)
    pub fn set_fill_rgb(&mut self, r: f32, g: f32, b: f32) {
        self.fill_color = Color::from_rgba(r, g, b, self.fill_alpha).unwrap_or(Color::BLACK);
    }

    /// Set stroke color from RGB values (0.0 - 1.0)
    pub fn set_stroke_rgb(&mut self, r: f32, g: f32, b: f32) {
        self.stroke_color = Color::from_rgba(r, g, b, self.stroke_alpha).unwrap_or(Color::BLACK);
    }

    /// Set fill color from grayscale (0.0 - 1.0)
    pub fn set_fill_gray(&mut self, gray: f32) {
        self.set_fill_rgb(gray, gray, gray);
    }

    /// Set stroke color from grayscale (0.0 - 1.0)
    pub fn set_stroke_gray(&mut self, gray: f32) {
        self.set_stroke_rgb(gray, gray, gray);
    }

    /// Set fill color from CMYK (0.0 - 1.0 each)
    pub fn set_fill_cmyk(&mut self, c: f32, m: f32, y: f32, k: f32) {
        // Simple CMYK to RGB conversion
        let r = (1.0 - c) * (1.0 - k);
        let g = (1.0 - m) * (1.0 - k);
        let b = (1.0 - y) * (1.0 - k);
        self.set_fill_rgb(r, g, b);
    }

    /// Set stroke color from CMYK (0.0 - 1.0 each)
    pub fn set_stroke_cmyk(&mut self, c: f32, m: f32, y: f32, k: f32) {
        let r = (1.0 - c) * (1.0 - k);
        let g = (1.0 - m) * (1.0 - k);
        let b = (1.0 - y) * (1.0 - k);
        self.set_stroke_rgb(r, g, b);
    }

    /// Apply a transformation to the CTM
    pub fn concat_ctm(&mut self, transform: Transform) {
        self.ctm = self.ctm.post_concat(transform);
    }

    /// Set line cap from PDF value (0=butt, 1=round, 2=square)
    pub fn set_line_cap(&mut self, cap: i32) {
        self.line_cap = match cap {
            0 => LineCap::Butt,
            1 => LineCap::Round,
            2 => LineCap::Square,
            _ => LineCap::Butt,
        };
    }

    /// Set line join from PDF value (0=miter, 1=round, 2=bevel)
    pub fn set_line_join(&mut self, join: i32) {
        self.line_join = match join {
            0 => LineJoin::Miter,
            1 => LineJoin::Round,
            2 => LineJoin::Bevel,
            _ => LineJoin::Miter,
        };
    }
}

/// Stack of graphics states for save/restore (q/Q operators)
#[derive(Debug, Default)]
pub struct GraphicsStateStack {
    stack: Vec<GraphicsState>,
    current: GraphicsState,
}

impl GraphicsStateStack {
    /// Create a new graphics state stack
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the current graphics state
    pub fn current(&self) -> &GraphicsState {
        &self.current
    }

    /// Get mutable reference to current graphics state
    pub fn current_mut(&mut self) -> &mut GraphicsState {
        &mut self.current
    }

    /// Save current state (q operator)
    pub fn save(&mut self) {
        self.stack.push(self.current.clone());
    }

    /// Restore previous state (Q operator)
    pub fn restore(&mut self) {
        if let Some(state) = self.stack.pop() {
            self.current = state;
        }
    }

    /// Reset to default state
    pub fn reset(&mut self) {
        self.stack.clear();
        self.current = GraphicsState::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let state = GraphicsState::default();
        assert_eq!(state.line_width, 1.0);
        assert_eq!(state.fill_alpha, 1.0);
        assert_eq!(state.stroke_alpha, 1.0);
    }

    #[test]
    fn test_color_rgb() {
        let mut state = GraphicsState::new();
        state.set_fill_rgb(1.0, 0.0, 0.0);
        // tiny_skia::Color uses f32 values (0.0-1.0)
        assert!((state.fill_color.red() - 1.0).abs() < 0.01);
        assert!(state.fill_color.green().abs() < 0.01);
        assert!(state.fill_color.blue().abs() < 0.01);
    }

    #[test]
    fn test_color_gray() {
        let mut state = GraphicsState::new();
        state.set_fill_gray(0.5);
        // Gray 0.5 = RGB(0.5, 0.5, 0.5)
        assert!((state.fill_color.red() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_state_stack() {
        let mut stack = GraphicsStateStack::new();

        stack.current_mut().line_width = 5.0;
        stack.save();

        stack.current_mut().line_width = 10.0;
        assert_eq!(stack.current().line_width, 10.0);

        stack.restore();
        assert_eq!(stack.current().line_width, 5.0);
    }

    #[test]
    fn test_line_cap_conversion() {
        let mut state = GraphicsState::new();

        state.set_line_cap(0);
        assert!(matches!(state.line_cap, LineCap::Butt));

        state.set_line_cap(1);
        assert!(matches!(state.line_cap, LineCap::Round));

        state.set_line_cap(2);
        assert!(matches!(state.line_cap, LineCap::Square));
    }
}
