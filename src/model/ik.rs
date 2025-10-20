use core::f32::consts::PI;
use libm::{acosf, atan2f, sqrtf};

const RAD_TO_DEG: f32 = 180f32 / PI;

pub struct Joint {
    length: f32,
}

impl Joint {
    pub fn new(length: f32) -> Self {
        Self { length }
    }
}

#[allow(unused)]
pub struct IkSolver {
    j0: Joint,
    j1: Joint,
    j2: Joint,
}

impl IkSolver {
    pub fn new(j0: Joint, j1: Joint, j2: Joint) -> Self {
        Self { j0, j1, j2 }
    }

    pub fn solve(&self, x: f32, y: f32, z: f32) -> Option<(f32, f32, f32)> {
        let mut x = x;
        let mut y = y;
        let mut a1 = 0.0;

        if z != 0.0 {
            let len = sqrtf((x * x) + (y * y) + (z * z));
            let xy_len = sqrtf((x * x) + (y * y));

            let z_angle = atan2f(z, xy_len);
            a1 = z_angle * RAD_TO_DEG;

            let mul = len / xy_len;
            x = x * mul;
            y = y * mul;
        }

        let (a2, a3) = self.solve_2d(x, y);

        if !(a1.is_finite() && a2.is_finite() && a3.is_finite()) {
            return None;
        }

        Some((a1, a2, a3))
    }

    fn solve_2d(&self, x: f32, y: f32) -> (f32, f32) {
        let start_to_end = sqrtf((x * x) + (y * y));

        let num = (start_to_end * start_to_end) + (self.j1.length * self.j1.length)
            - (self.j2.length * self.j2.length);
        let denom = 2.0 * start_to_end * self.j1.length;
        let a1_degree = acosf(num / denom) * RAD_TO_DEG;
        let a1_offset = atan2f(x, y) * RAD_TO_DEG;

        let a1 = a1_degree + a1_offset;
        let a2 = 180.0 - (a1_degree * 2.0);

        (a1, a2)
    }
}
