#![feature(portable_simd)]
use std::simd::*;

use std::time::Instant;

// use nannou::prelude::*;

const WIDTH: usize = 1000;
const HEIGHT: usize = 1000;
const MAX_ITERS: usize = 200;

fn main() {
    let mut iters = [0; WIDTH * HEIGHT];

    let scale = 1.0;
    let x_off = 0.0;
    let y_off = 0.0;

    let scale_vec = f64x4::splat(scale);
    let x_off_vec = f64x4::splat(x_off);

    let start = Instant::now();

    for row in 0..HEIGHT {
        let mut n = i64x4::splat(1);
        let mut done_mask = mask64x4::splat(false);

        for x in (0..WIDTH).step_by(4) {
            let xs = f64x4::from_array([x as f64, (x + 1) as f64, (x + 2) as f64, (x + 3) as f64]);
            let mut zr = xs / scale_vec + x_off_vec;
            let mut zi = f64x4::splat(row as f64 / scale + y_off);

            let cr = zr.clone();
            let ci = zi.clone();

            for _ in 0..MAX_ITERS {
                // Square z and add c
                let zr_new = zr * zr - zi * zi + cr;
                let zi_new = f64x4::splat(2.0) * zr * zi + ci;

                // Check if absolute value is less than 2
                let squared_abs = zr_new * zr_new + zi_new * zi_new;
                let now_done = squared_abs >= f64x4::splat(4.0);
                done_mask |= now_done;

                // Increment n by one
                let mut to_add = i64x4::splat(1);
                to_add &= done_mask.to_int();
                n += to_add;

                // Check if all done
                let reached_max = n.lanes_ge(i64x4::splat(MAX_ITERS as i64));
                done_mask |= reached_max;

                zr = zr_new;
                zi = zi_new;
            }

            // Save calculated values in iters array
            iters[row * WIDTH + x] = n[0];
            iters[row * WIDTH + x + 1] = n[1];
            iters[row * WIDTH + x + 2] = n[2];
            iters[row * WIDTH + x + 3] = n[3];
        }
    }

    let elapsed = start.elapsed();

    println!("Finished in {} milliseconds.", elapsed.as_millis());
}
