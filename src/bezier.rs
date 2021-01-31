use num_traits::ToPrimitive;
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
    pub rate_coef: f32,
    pub rate_1: f32,
    pub rate_2: f32,
    pub p_0: [f32; 3],
    pub p_1: [f32; 3],
    pub p_2: [f32; 3],
    pub p_3: [f32; 3],
}

pub struct BezierTrajectory {
    t: f32,
    rate: f32,
    loop_enabled: bool,
    curves: Vec<Curve>,
    t_duration: f32,
    t_hint: Vec<f32>,
}

impl BezierTrajectory {
    pub fn new(loop_enabled: bool, curves: Vec<Curve>) -> Self {
        let (t_duration, t_hint) = calc_t_duration_hint(&curves);
        Self {
            t: 0.,
            rate: 0.1,
            curves,
            loop_enabled,
            t_hint,
            t_duration,
        }
    }

    pub fn set_loop(&mut self, loop_enabled: bool) {
        self.loop_enabled = loop_enabled;
    }

    /// duration: ms
    /// stride: ms / count
    pub fn set_rate_by_time(&mut self, duration: f32, stride: f32) {
        self.rate = self.t_duration / duration * stride;
    }

    pub fn calc_point(&self, t: f32, rate: f32, loop_enabled: bool) -> Option<([f32; 3], f32)> {
        calc_point(&self.curves, t, rate, loop_enabled)
    }

    pub fn calc_current_point(&self) -> Option<([f32; 3], f32)> {
        self.calc_point(self.t, self.rate, self.loop_enabled)
    }

    pub fn next(&mut self) -> Option<[f32; 3]> {
        match self.calc_current_point() {
            Some((out, next_t)) => {
                self.t = next_t;
                Some(out)
            }
            None => None,
        }
    }

    pub fn calc_point_by_time(&self, duration: f32, time: f32) -> Option<[f32; 3]> {
        let raw = (time / duration * self.t_duration).max(0.);
        let t = {
            let mut base = 0.;
            let t = self
                .t_hint
                .iter()
                .zip(self.curves.iter())
                .find_map(|(hint, curve)| {
                    if raw < *hint {
                        Some((raw - base) * curve.rate_coef)
                    } else {
                        base = *hint;
                        None
                    }
                });
            match t {
                Some(t) => t,
                None => return None,
            }
        };
        self.calc_point(t, 0., false).and_then(|(out, _)| Some(out))
    }
}

fn calc_point(
    curves: &Vec<Curve>,
    t: f32,
    rate: f32,
    loop_enabled: bool,
) -> Option<([f32; 3], f32)> {
    let mut next_t = t;
    let (u, i) = {
        let f = t.floor().max(0.);
        let i = f.to_u32().unwrap() as usize;
        let u = t - f;
        let i = if i < curves.len() {
            i
        } else if loop_enabled {
            next_t = u;
            0
        } else {
            return None;
        };
        (u, i)
    };
    let curve = curves.get(i).unwrap();
    next_t += curve.rate_coef * rate;
    let u = {
        let mut o = [0.];
        bezier(u, &mut o, &[0.], &[curve.rate_1], &[curve.rate_2], &[1.]);
        o[0]
    };
    let mut out = [0., 0., 0.];
    bezier(u, &mut out, &curve.p_0, &curve.p_1, &curve.p_2, &curve.p_3);
    Some((out, next_t))
}

fn calc_t_duration_hint(curves: &Vec<Curve>) -> (f32, Vec<f32>) {
    let mut acc: f32 = 0.;
    let t_hint: Vec<f32> = curves
        .iter()
        .map(|curve| {
            acc += 1. / curve.rate_coef;
            acc
        })
        .collect();
    (acc, t_hint)
}
