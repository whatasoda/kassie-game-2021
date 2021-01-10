use crate::impls::matrix::ViewMatrix;
use crate::ConvertArrayView;
use webgl_matrix::{Mat4, Matrix, ProjectionMatrix, Vec3};

#[repr(C)]
pub struct Camera {
    pub vp_matrix: Mat4,
}
impl ConvertArrayView for Camera {}

pub struct CameraController {
    pub camera: Camera,
    projection: Projection,
    pub view: View,
}
impl Default for CameraController {
    fn default() -> Self {
        let projection = Projection::default();
        let view = View::default();
        let vp_matrix = Mat4::identity();
        let mut s = Self {
            camera: Camera { vp_matrix },
            projection,
            view,
        };
        s.refresh();
        s
    }
}
impl CameraController {
    pub fn refresh(&mut self) {
        self.projection.refresh();
        self.view.refresh();
        self.view.matrix.copy_to(&mut self.camera.vp_matrix);
        self.camera.vp_matrix.mul(&self.projection.matrix);
    }
}

pub struct View {
    pub matrix: Mat4,
    pub position: Vec3,
    up: Vec3,
    pub direction: Vec3,
}
impl Default for View {
    fn default() -> Self {
        let up = [0., 1., 0.];
        let direction = [0., 0., -1.];
        let position = [0., 0., 0.];
        Self {
            matrix: Mat4::create_view(&position, &up, &direction),
            up,
            direction,
            position,
        }
    }
}
impl View {
    fn refresh(&mut self) {
        self.matrix = Mat4::create_view(&self.position, &self.up, &self.direction);
    }
}

pub struct Projection {
    matrix: Mat4,
    fov_y: f32,
    aspect_retio: f32,
    near: f32,
    far: f32,
}
impl Default for Projection {
    fn default() -> Self {
        let fov_y = 40.;
        let aspect_retio = 1.;
        let near = 0.001;
        let far = 100000.;
        Self {
            matrix: Mat4::create_perspective(fov_y, aspect_retio, near, far),
            fov_y,
            aspect_retio,
            near,
            far,
        }
    }
}
impl Projection {
    fn refresh(&mut self) {
        self.matrix = Mat4::create_perspective(self.fov_y, self.aspect_retio, self.near, self.far);
    }
}
