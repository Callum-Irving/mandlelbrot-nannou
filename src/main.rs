#![feature(portable_simd)]
use std::simd::*;

use nannou::prelude::*;

const WIDTH: usize = 1000;
const HEIGHT: usize = 1000;
const MAX_ITERS: usize = 200;

fn main() {
    let mut iters = [0; WIDTH * HEIGHT];

    let scale = 1.0;
    let xOff = 0.0;
    let yOff = 0.0;

    let scaleVec = f64x4::splat(scale);
    let xOffVec = f64x4::splat(xOff);

    for row in 0..HEIGHT {
        let mut n = i64x4::splat(1);
        let mut done_mask = mask64x4::splat(false);

        for x in (0..WIDTH).step_by(4) {
            let xs = f64x4::from_array([x as f64, (x + 1) as f64, (x + 2) as f64, (x + 3) as f64]);
            let mut zr = xs / scaleVec + xOffVec;
            let mut zi = f64x4::splat(row as f64 / scale + yOff);

            for _ in 0..MAX_ITERS {
                let zr_new = zr * zr - zi * zi;
                let zi_new = f64x4::splat(2.0) * zr * zi;

                let squared_abs = zr_new * zr_new + zi_new * zi_new;
                let now_done = squared_abs >= f64x4::splat(4.0);
                done_mask |= now_done;

                let mut to_add = i64x4::splat(1);
                to_add &= done_mask.to_int();
                n += to_add;

                zr = zr_new;
                zi = zi_new;
            }
        }
    }
}
