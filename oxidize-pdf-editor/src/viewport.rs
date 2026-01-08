/// Viewport manages the current view state of a PDF document
#[derive(Debug, Clone)]
pub struct Viewport {
    current_page: usize,
    page_count: usize,
    zoom: f32,
    pan_x: f32,
    pan_y: f32,
}

impl Viewport {
    const MIN_ZOOM: f32 = 0.25;
    const MAX_ZOOM: f32 = 4.0;
    const ZOOM_STEP: f32 = 0.25;
    const DEFAULT_ZOOM: f32 = 1.0;

    pub fn new(page_count: usize) -> Self {
        Self {
            current_page: 0,
            page_count,
            zoom: Self::DEFAULT_ZOOM,
            pan_x: 0.0,
            pan_y: 0.0,
        }
    }

    pub fn current_page(&self) -> usize {
        self.current_page
    }

    pub fn set_page(&mut self, page: usize) {
        if page < self.page_count {
            self.current_page = page;
            // Reset pan when changing pages
            self.pan_x = 0.0;
            self.pan_y = 0.0;
        }
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom + Self::ZOOM_STEP).min(Self::MAX_ZOOM);
    }

    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom - Self::ZOOM_STEP).max(Self::MIN_ZOOM);
    }

    pub fn reset_zoom(&mut self) {
        self.zoom = Self::DEFAULT_ZOOM;
        self.pan_x = 0.0;
        self.pan_y = 0.0;
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(Self::MIN_ZOOM, Self::MAX_ZOOM);
    }

    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.pan_x += dx;
        self.pan_y += dy;
    }

    pub fn pan_position(&self) -> (f32, f32) {
        (self.pan_x, self.pan_y)
    }

    pub fn next_page(&mut self) -> bool {
        if self.current_page + 1 < self.page_count {
            self.current_page += 1;
            self.pan_x = 0.0;
            self.pan_y = 0.0;
            true
        } else {
            false
        }
    }

    pub fn previous_page(&mut self) -> bool {
        if self.current_page > 0 {
            self.current_page -= 1;
            self.pan_x = 0.0;
            self.pan_y = 0.0;
            true
        } else {
            false
        }
    }
}
