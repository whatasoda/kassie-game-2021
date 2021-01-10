use webgl_matrix::{Vec3, Vector};

pub trait Cross {
    fn cross(a: &[f32], b: &[f32]) -> Vec3;
}

impl Cross for Vec3 {
    fn cross(a: &[f32], b: &[f32]) -> Self {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    }
}
