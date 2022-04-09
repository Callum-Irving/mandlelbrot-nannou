#![feature(portable_simd)]
use std::cmp::max;
use std::simd::*;

use minifb::{Key, KeyRepeat, Window, WindowOptions};
use minifb::{MouseButton, MouseMode};

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;

fn main() {
    let mut max_iters: usize = 200;
    let mut buf = vec![0; WIDTH * HEIGHT];
    let mut scale = 800.0;
    let mut x_off = -0.750222;
    let mut y_off = -0.031161;

    // For panning
    let mut prev_mouse_pos: Option<(f32, f32)> = None;

    // Precompute colours
    let mut colours = vec![];
    colours = precompute_complex(max_iters + 1, colours);

    // Create window
    let mut window =
        Window::new("Mandelbrot Set", WIDTH, HEIGHT, WindowOptions::default()).unwrap();

    // Event loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Draw Mandelbrot set
        draw_mandelbrot_vectorized(&mut buf, scale, x_off, y_off, &colours, max_iters);
        window.update_with_buffer(&buf, WIDTH, HEIGHT).unwrap();

        // Panning
        if window.get_mouse_down(MouseButton::Left) {
            let current_pos = window.get_mouse_pos(MouseMode::Pass).unwrap();
            if let Some(prev_mouse_pos) = prev_mouse_pos {
                x_off += (prev_mouse_pos.0 - current_pos.0) as f64 / scale;
                y_off += (prev_mouse_pos.1 - current_pos.1) as f64 / scale;
            }
            prev_mouse_pos = Some(current_pos);
        } else {
            prev_mouse_pos = None;
        }

        // Zooming
        if let Some(scroll) = window.get_scroll_wheel() {
            let mouse_pos = window.get_mouse_pos(MouseMode::Pass).unwrap();
            let prev_x = mouse_pos.0 as f64 / scale + x_off;
            let prev_y = mouse_pos.1 as f64 / scale + y_off;
            if scroll.1 > 0.0 {
                scale *= 1.1;
            } else {
                scale /= 1.1;
            }
            let curr_x = mouse_pos.0 as f64 / scale + x_off;
            let curr_y = mouse_pos.1 as f64 / scale + y_off;
            x_off += prev_x - curr_x;
            y_off += prev_y - curr_y;
        }

        // Changing number of iterations
        window
            .get_keys_pressed(KeyRepeat::Yes)
            .iter()
            .for_each(|key| match key {
                Key::Up => max_iters += 10,
                Key::Down => max_iters = max(max_iters - 10, 10),
                _ => (),
            });
        colours = precompute_complex(max_iters + 1, colours);
    }
}

#[inline]
fn from_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    r << 16 | g << 8 | b
}

fn precompute_complex(iters: usize, mut colours: Vec<u32>) -> Vec<u32> {
    let old_len = colours.len();
    let a = 0.1;
    if iters > old_len {
        colours.reserve(iters - old_len);
        for i in old_len..iters {
            let r = (0.5 * (a * i as f32).sin() + 0.5) * 255.0;
            let g = (0.5 * (a * i as f32 + 2.094).sin() + 0.5) * 255.0;
            let b = (0.5 * (a * i as f32 + 4.188).sin() + 0.5) * 255.0;
            colours.push(from_rgb(r as u8, g as u8, b as u8));
        }
    }
    colours
}

const TWO: f64x4 = f64x4::splat(2.0);
const FOUR: f64x4 = f64x4::splat(4.0);
fn draw_mandelbrot_vectorized(
    buffer: &mut [u32],
    scale: f64,
    x_off: f64,
    y_off: f64,
    colours: &[u32],
    max_iters: usize,
) {
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

            for _ in 0..max_iters {
                // Square z and add c
                let new_zr = zr * zr - zi * zi + cr;
                let new_zi = zr * zi * TWO + ci;
                zr = new_zr;
                zi = new_zi;

                // Check if absolute value is less than 2
                let squared_abs = zr * zr + zi * zi;

                // True means we need to increment
                let not_diverged = squared_abs.lanes_lt(FOUR);

                // Increment n by one
                let to_add = i64x4::splat(1) & not_diverged.to_int();

                n += to_add;

                // If all diverged
                if not_diverged.to_int().reduce_min() == 0 {
                    break;
                }
            }

            // Save calculated values in buffer as colours
            buffer[y * WIDTH + x] = colours[n[0] as usize];
            buffer[y * WIDTH + x + 1] = colours[n[1] as usize];
            buffer[y * WIDTH + x + 2] = colours[n[2] as usize];
            buffer[y * WIDTH + x + 3] = colours[n[3] as usize];
        }
    }
}
