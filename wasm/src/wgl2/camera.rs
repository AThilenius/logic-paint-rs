use glam::{Mat4, Quat, Vec2, Vec3};

pub struct Camera {
    pub translation: Vec2,
    pub scale: f32,
    pixel_ratio: f32,
    width: f32,
    height: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            translation: Vec2::ZERO,
            scale: 1.0,
            pixel_ratio: 1.0,
            width: 1.0,
            height: 1.0,
        }
    }
}

impl Camera {
    pub fn update(&mut self, pixel_ratio: f32, width: f32, height: f32) {
        self.pixel_ratio = pixel_ratio;
        self.width = width;
        self.height = height;
    }

    pub fn get_view_proj_matrix(&self) -> Mat4 {
        // Use a DPI-like scaling, so that the scale doesn't change with screen size, only the
        // amount visible.
        let scale = 720.0 * 2.0 * self.pixel_ratio;
        let w = self.width / scale;
        let h = self.height / scale;
        let proj_mat = Mat4::orthographic_rh(-w, w, -h, h, 0.0, 1.0);

        let view_mat = Mat4::from_scale_rotation_translation(
            Vec3::ONE * self.scale,
            Quat::IDENTITY,
            Vec3::new(self.translation.x, self.translation.y, 0.0),
        )
        .inverse();

        proj_mat * view_mat
    }
}
