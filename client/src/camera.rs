
#[derive(Debug, Default, Clone, Copy)]
pub struct Camera {
    /// Center of the camera.
    position: na::Vector2<f32>,
    zoom: f32,
}
impl Camera {
    pub fn get_visible_rect(&self, screen_size: na::Vector2<f32>) -> parry2d::bounding_volume::AABB {
        parry2d::bounding_volume::AABB::from_half_extents(
            self.position.into(),
            screen_size * 0.5 * self.zoom,
        )
    }

    /// Set the camera's position.
    pub fn set_position(&mut self, position: na::Vector2<f32>) {
        self.position = position;
    }

    /// Get the camera's position.
    #[must_use]
    pub fn position(&self) -> na::Vector2<f32> {
        self.position
    }

    /// Set the camera's zoom.
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom;
    }

    /// Get the camera's zoom.
    #[must_use]
    pub fn zoom(&self) -> f32 {
        self.zoom
    }
}
