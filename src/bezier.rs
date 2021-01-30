use num_traits::ToPrimitive;

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

struct Curve {
    base_rate: f32,
    rate_1: f32,
    rate_2: f32,
    p_0: [f32; 3],
    p_1: [f32; 3],
    p_2: [f32; 3],
    p_3: [f32; 3],
}

struct Trajectory {
    t: f32,
    rate: f32,
    pub loop_enabled: bool,
    curves: Vec<Curve>,
}

impl Trajectory {
    fn next<F>(&mut self) -> Option<[f32; 3]>
    where
        F: FnOnce(f32) -> [f32; 3],
    {
        let (u, idx) = {
            let f = self.t.floor().max(0.);
            let idx = f.to_u32().unwrap() as usize;
            let u = self.t - f;
            (
                u,
                if idx < self.curves.len() {
                    idx
                } else if self.loop_enabled {
                    self.t = u;
                    0
                } else {
                    return None;
                },
            )
        };
        let curve = self.curves.get(idx).unwrap();
        self.t += curve.base_rate * self.rate;
        let u = {
            let mut o = [0.];
            bezier(u, &mut o, &[0.], &[curve.rate_1], &[curve.rate_2], &[1.]);
            o[0]
        };
        let mut out = [0., 0., 0.];
        bezier(u, &mut out, &curve.p_0, &curve.p_1, &curve.p_2, &curve.p_3);
        Some(out)
    }
}
