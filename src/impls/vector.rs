use webgl_matrix::{Vec3, Vector};

pub trait Cross {
    fn cross(&self, b: &[f32]) -> Vec3;
}

impl Cross for Vec3 {
    fn cross(&self, b: &[f32]) -> Vec3 {
        [
            self[1] * b[2] - self[2] * b[1],
            self[2] * b[0] - self[0] * b[2],
            self[0] * b[1] - self[1] * b[0],
        ]
    }
}

pub trait Normalize {
    fn normalize(&self) -> Vec3;
}

impl Normalize for Vec3 {
    fn normalize(&self) -> Vec3 {
        let mag = 1. / self.mag();
        [self[0] * mag, self[1] * mag, self[2] * mag]
    }
}
