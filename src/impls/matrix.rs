use std::f32::EPSILON;
use webgl_matrix::{Mat4, Matrix};

pub trait ViewMatrix {
    fn create_view(position: &[f32], up: &[f32], direction: &[f32]) -> Mat4;
}

impl ViewMatrix for Mat4 {
    fn create_view(eye: &[f32], up: &[f32], direction: &[f32]) -> Self {
        let mut len;
        let eyex = eye[0];
        let eyey = eye[1];
        let eyez = eye[2];
        let upx = up[0];
        let upy = up[1];
        let upz = up[2];
        let mut z0 = -direction[0];
        let mut z1 = -direction[1];
        let mut z2 = -direction[2];
        if z0.abs() < EPSILON && z1.abs() < EPSILON && z2.abs() < EPSILON {
            return Mat4::identity();
        }

        len = 1. / (z0.powf(2.) + z1.powf(2.) + z2.powf(2.));
        z0 *= len;
        z1 *= len;
        z2 *= len;

        let mut x0 = upy * z2 - upz * z1;
        let mut x1 = upz * z0 - upx * z2;
        let mut x2 = upx * z1 - upy * z0;
        len = x0.powf(2.) + x1.powf(2.) + x2.powf(2.);
        if len < EPSILON {
            x0 = 0.;
            x1 = 0.;
            x2 = 0.;
        } else {
            len = 1. / len;
            x0 *= len;
            x1 *= len;
            x2 *= len;
        }

        let mut y0 = z1 * x2 - z2 * x1;
        let mut y1 = z2 * x0 - z0 * x2;
        let mut y2 = z0 * x1 - z1 * x0;
        len = y0.powf(2.) + y1.powf(2.) + y2.powf(2.);
        if len < EPSILON {
            y0 = 0.;
            y1 = 0.;
            y2 = 0.;
        } else {
            len = 1. / len;
            y0 *= len;
            y1 *= len;
            y2 *= len;
        }
        [
            x0,
            y0,
            z0,
            0.,
            x1,
            y1,
            z1,
            0.,
            x2,
            y2,
            z2,
            0.,
            -(x0 * eyex + x1 * eyey + x2 * eyez),
            -(y0 * eyex + y1 * eyey + y2 * eyez),
            -(z0 * eyex + z1 * eyey + z2 * eyez),
            1.,
        ]
    }
}
