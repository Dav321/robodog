use core::f32::consts::PI;
use embassy_rp::rom_data::float_funcs::{fatan2, fsqrt};
use libm::acosf;

pub struct Joint {
    length: f32,
}

impl Joint {
    pub fn new(length: f32) -> Self {
        Self { length }
    }
}

pub struct IkSolver {
    j1: Joint,
    j2: Joint,
}

impl IkSolver {
    pub fn new(j1: Joint, j2: Joint) -> Self {
        Self { j1, j2 }
    }

    pub fn solve(&self, x: f32, y: f32) -> (u8, u8) {
        const RAD_TO_DEG: f32 = 180f32 / PI;

        let start_to_end = fsqrt(x * x + y * y);

        let z = (start_to_end * start_to_end) + (self.j1.length * self.j1.length)
            - (self.j2.length * self.j2.length);
        let n = 2f32 * start_to_end * self.j1.length;
        let d = acosf(z / n) + fatan2(x, y);

        let a2 = (d * RAD_TO_DEG) as u8;
        let a1 = (180 - a2) / 2;

        (a1, a2)
    }
}
