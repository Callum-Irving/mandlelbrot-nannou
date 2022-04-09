#![feature(portable_simd)]
use std::time::{Duration, Instant};
use std::{simd::*, thread};

use minifb::{Window, WindowOptions};

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;
const MAX_ITERS: i64 = 500;

fn main() {
    let mut buf = vec![0; WIDTH * HEIGHT];
    let scale = 1000.0;
    let x_off = -0.750222;
    let y_off = -0.031161;
    let mut colours = [0; MAX_ITERS as usize + 1];
    colours[MAX_ITERS as usize] = 0x00FFFFFF;

    let start = Instant::now();
    draw_mandelbrot(&mut buf, scale, x_off, y_off, colours);
    let elapsed = start.elapsed();
    println!("Finished in {} milliseconds.", elapsed.as_millis());

    let start = Instant::now();
    draw_mandelbrot_vectorized(&mut buf, scale, x_off, y_off, colours);
    let elapsed = start.elapsed();
    println!("Finished in {} milliseconds.", elapsed.as_millis());

    let mut window =
        Window::new("Mandelbrot Set", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    window.update_with_buffer(&buf, WIDTH, HEIGHT).unwrap();

    thread::sleep(Duration::from_millis(2000));
}

fn from_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    r << 16 | g << 8 | b
}

fn draw_mandelbrot(
    buffer: &mut [u32],
    scale: f64,
    x_off: f64,
    y_off: f64,
    colours: [u32; MAX_ITERS as usize + 1],
) {
    let mut iters = [0; WIDTH * HEIGHT];

    for y in 0..HEIGHT {
        let ci = y as f64 / scale + y_off;
        for x in 0..WIDTH {
            let cr = x as f64 / scale + x_off;

            let mut zr = 0.0;
            let mut zi = 0.0;

            let mut n: usize = 0;

            for _ in 0..MAX_ITERS {
                let new_zr = zr * zr - zi * zi + cr;
                let new_zi = zr * zi * 2.0 + ci;
                zr = new_zr;
                zi = new_zi;

                let squared_abs = zr * zr + zi * zi;
                if squared_abs >= 4.0 {
                    break;
                }
                n += 1;
            }

            iters[y * WIDTH + x] = n;
        }
    }

    for (i, pixel) in buffer.iter_mut().enumerate() {
        *pixel = colours[iters[i]];
    }
}

fn draw_mandelbrot_vectorized(
    buffer: &mut [u32],
    scale: f64,
    x_off: f64,
    y_off: f64,
    colours: [u32; MAX_ITERS as usize + 1],
) {
    let mut iters = [0; WIDTH * HEIGHT];

    let scale_vec = f64x4::splat(scale);
    let x_off_vec = f64x4::splat(x_off);

    for y in 0..HEIGHT {
        let ci = f64x4::splat(y as f64 / scale + y_off);

        for x in (0..WIDTH).step_by(4) {
            let mut n = i64x4::splat(0);
            let xs = f64x4::from_array([x as f64, (x + 1) as f64, (x + 2) as f64, (x + 3) as f64]);
            let cr = xs / scale_vec + x_off_vec;

            let mut zr = f64x4::splat(0.0);
            let mut zi = f64x4::splat(0.0);

            for _ in 0..MAX_ITERS {
                // Square z and add c
                let new_zr = zr * zr - zi * zi + cr;
                let new_zi = f64x4::splat(2.0) * zr * zi + ci;
                zr = new_zr;
                zi = new_zi;

                // Check if absolute value is less than 2
                let squared_abs = zr * zr + zi * zi;

                // True means we need to increment
                let not_diverged = squared_abs.lanes_lt(f64x4::splat(4.0));

                // Increment n by one
                let to_add = i64x4::splat(1) & not_diverged.to_int();

                n += to_add;

                // If all diverged
                if not_diverged.to_int().reduce_min() == 0 {
                    break;
                }
            }

            // Save calculated values in iters array
            iters[y * WIDTH + x] = n[0] as usize;
            iters[y * WIDTH + x + 1] = n[1] as usize;
            iters[y * WIDTH + x + 2] = n[2] as usize;
            iters[y * WIDTH + x + 3] = n[3] as usize;
        }
    }

    for (i, pixel) in buffer.iter_mut().enumerate() {
        *pixel = colours[iters[i]];
    }
}
