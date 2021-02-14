use std::iter::Iterator;

#[inline]
fn prepare_t(t: f32) -> (f32, f32, f32, f32) {
    let t2 = t * t;
    let u = 1. - t;
    let u2 = u * u;
    (
        u * u2,      //
        3. * t * u2, //
        3. * u * t2, //
        t * t2,      //
    )
}

#[inline]
pub fn bezier_scalar(t: f32, p: (f32, f32, f32, f32)) -> f32 {
    let t = prepare_t(t);
    t.0 * p.0 + t.1 * p.1 + t.2 * p.2 + t.3 * p.3
}

#[inline]
pub fn bezier_slice(t: f32, out: &mut [f32], p_0: &[f32], p_1: &[f32], p_2: &[f32], p_3: &[f32]) {
    let t = prepare_t(t);

    let p_01 = p_0.iter().zip(p_1.iter());
    let p_23 = p_2.iter().zip(p_3.iter());

    for (o, ((p_0, p_1), (p_2, p_3))) in out.iter_mut().zip(p_01.zip(p_23)) {
        *o = t.0 * p_0 + t.1 * p_1 + t.2 * p_2 + t.3 * p_3;
    }
}

pub trait ParametricCurveSequence {
    type Config;
    fn new(config: Self::Config) -> Self;
    fn duration(&self) -> f32;
    fn calc_point(&self, t: f32, loop_enabled: bool) -> Option<[f32; 3]>;
}

pub struct Curve {
    pub t_duration: f32,
    pub t_p: Option<(f32, f32)>,
    pub p_0: [f32; 3],
    pub p_1: [f32; 3],
    pub p_2: [f32; 3],
    pub p_3: [f32; 3],
}

pub struct BezierSequence {
    curves: Vec<Curve>,
    t_duration: f32,
}

impl ParametricCurveSequence for BezierSequence {
    type Config = Vec<Curve>;

    fn new(curves: Self::Config) -> Self {
        let mut t_duration = 0.;
        for curve in &curves {
            t_duration += curve.t_duration;
        }
        Self { curves, t_duration }
    }

    fn duration(&self) -> f32 {
        self.t_duration
    }

    fn calc_point(&self, t: f32, loop_enabled: bool) -> Option<[f32; 3]> {
        let t = if t <= self.t_duration {
            t
        } else if loop_enabled {
            t % self.t_duration
        } else {
            return None;
        };

        let mut base = 0.0;
        let curve = self.curves.iter().find_map(|curve| {
            let next_base = base + curve.t_duration;
            if t <= next_base {
                let t = (t - next_base) / curve.t_duration;
                Some((t, curve))
            } else {
                base = next_base;
                None
            }
        });
        curve.and_then(|(t, curve)| {
            let t = match curve.t_p {
                Some((t_p_1, t_p_2)) => bezier_scalar(t, (0., t_p_1, t_p_2, 1.)),
                None => t,
            };
            let mut out = [0., 0., 0.];
            bezier_slice(t, &mut out, &curve.p_0, &curve.p_1, &curve.p_2, &curve.p_3);
            Some(out)
        })
    }
}
