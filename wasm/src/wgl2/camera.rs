use glam::{Mat4, Quat, Vec2, Vec3};

pub struct Camera {
    pub translation: Vec2,
    pub scale: f32,
    width: u32,
    height: u32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            translation: Default::default(),
            scale: 1.0,
            width: 1,
            height: 1,
        }
    }
}

impl Camera {
    pub fn update(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn get_view_proj_matrix(&self) -> Mat4 {
        let proj_mat = if self.width > self.height {
            let aspect = self.width as f32 / self.height as f32;
            Mat4::orthographic_rh(-aspect, aspect, -1.0, 1.0, 0.0, 1.0)
        } else {
            let aspect = self.height as f32 / self.width as f32;
            Mat4::orthographic_rh(-1.0, 1.0, -aspect, aspect, 0.0, 1.0)
        };

        let view_mat = Mat4::from_scale_rotation_translation(
            Vec3::ONE * self.scale,
            Quat::IDENTITY,
            Vec3::new(self.translation.x, self.translation.y, 0.0),
        )
        .inverse();

        proj_mat * view_mat
    }
}
