use std::iter::Iterator;

#[inline]
pub fn bezier(t: f32, out: &mut [f32], p_0: &[f32], p_1: &[f32], p_2: &[f32], p_3: &[f32]) {
    let t2 = t * t;
    let u = 1. - t;
    let u2 = u * u;

    let t_0 = u * u2;
    let t_1 = 3. * t * u2;
    let t_2 = 3. * u * t2;
    let t_3 = t * t2;

    for (o, p_0) in out.iter_mut().zip(p_0.iter()) {
        *o = p_0 * t_0;
    }
    for (o, p_1) in out.iter_mut().zip(p_1.iter()) {
        *o += p_1 * t_1;
    }
    for (o, p_2) in out.iter_mut().zip(p_2.iter()) {
        *o += p_2 * t_2;
    }
    for (o, p_3) in out.iter_mut().zip(p_3.iter()) {
        *o += p_3 * t_3;
    }
}

pub struct Curve {
    pub t_duration: f32,
    pub t_p: Option<(f32, f32)>,
    pub p_0: [f32; 3],
    pub p_1: [f32; 3],
    pub p_2: [f32; 3],
    pub p_3: [f32; 3],
}

pub struct BezierGroup {
    curves: Vec<Curve>,
    t_duration: f32,
}

impl BezierGroup {
    pub fn new(curves: Vec<Curve>) -> Self {
        let mut t_duration = 0.;
        for curve in curves {
            t_duration += curve.t_duration;
        }
        Self { curves, t_duration }
    }

    pub fn calc_point(&self, t: f32, loop_enabled: bool) -> Option<[f32; 3]> {
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
                Some((t_p_1, t_p_2)) => {
                    let mut o = [0.];
                    bezier(t, &mut o, &[0.], &[t_p_1], &[t_p_2], &[1.]);
                    o[0]
                }
                None => t,
            };
            let mut out = [0., 0., 0.];
            bezier(t, &mut out, &curve.p_0, &curve.p_1, &curve.p_2, &curve.p_3);
            Some(out)
        })
    }
}
