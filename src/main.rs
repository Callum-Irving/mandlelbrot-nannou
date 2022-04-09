#![feature(portable_simd)]
use std::cmp::max;
use std::simd::*;

use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;

fn main() {
    let mut max_iters: usize = 200;
    let mut scale = 800.0;
    let mut x_off = -0.750222;
    let mut y_off = -0.031161;

    // Window stuff
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Mandelbrot Set")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture).unwrap()
    };

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            draw_mandelbrot_vectorized(
                pixels.get_frame(),
                scale,
                x_off,
                y_off,
                // &colours,
                max_iters,
            );
            if pixels
                .render()
                .map_err(|e| eprintln!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Get mouse position info
            let (mouse_cell, mouse_prev_cell) = input
                .mouse()
                .map(|(mx, my)| {
                    let (dx, dy) = input.mouse_diff();
                    let prev_x = mx - dx;
                    let prev_y = my - dy;

                    let (mx_i, my_i) = pixels
                        .window_pos_to_pixel((mx, my))
                        .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                    let (px_i, py_i) = pixels
                        .window_pos_to_pixel((prev_x, prev_y))
                        .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                    (
                        (mx_i as isize, my_i as isize),
                        (px_i as isize, py_i as isize),
                    )
                })
                .unwrap_or_default();

            // Pan
            if input.mouse_held(0) {
                x_off -= (mouse_cell.0 - mouse_prev_cell.0) as f64 / scale;
                y_off -= (mouse_cell.1 - mouse_prev_cell.1) as f64 / scale;
            }

            // Zoom
            if input.scroll_diff() != 0.0 {
                let prev_x = mouse_cell.0 as f64 / scale - x_off;
                let prev_y = mouse_cell.1 as f64 / scale - y_off;

                if input.scroll_diff() > 0.0 {
                    scale *= 1.2 * input.scroll_diff() as f64;
                } else {
                    scale /= 1.2 * input.scroll_diff().abs() as f64;
                }

                let new_x = mouse_cell.0 as f64 / scale - x_off;
                let new_y = mouse_cell.1 as f64 / scale - y_off;
                x_off += prev_x - new_x;
                y_off += prev_y - new_y;
            }

            // Change number of iterations
            if input.key_pressed(VirtualKeyCode::Up) {
                max_iters += 10;
            }

            if input.key_pressed(VirtualKeyCode::Down) {
                max_iters = max(10, max_iters - 10);
            }

            window.request_redraw();
        }
    });
}

fn compute_colour(n: i64) -> (u8, u8, u8) {
    let a = 0.1;
    let r = (0.5 * (a * n as f32).sin() + 0.5) * 255.0;
    let g = (0.5 * (a * n as f32 + 2.094).sin() + 0.5) * 255.0;
    let b = (0.5 * (a * n as f32 + 4.188).sin() + 0.5) * 255.0;
    (r as u8, g as u8, b as u8)
}

const TWO: f64x4 = f64x4::splat(2.0);
const FOUR: f64x4 = f64x4::splat(4.0);
fn draw_mandelbrot_vectorized(
    buffer: &mut [u8],
    scale: f64,
    x_off: f64,
    y_off: f64,
    // colours: &[u32],
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
            let (r, g, b) = compute_colour(n[0]);
            buffer[(y * WIDTH + x) * 4] = r;
            buffer[(y * WIDTH + x) * 4 + 1] = g;
            buffer[(y * WIDTH + x) * 4 + 2] = b;
            buffer[(y * WIDTH + x) * 4 + 3] = 255;

            let (r, g, b) = compute_colour(n[1]);
            buffer[(y * WIDTH + x + 1) * 4] = r;
            buffer[(y * WIDTH + x + 1) * 4 + 1] = g;
            buffer[(y * WIDTH + x + 1) * 4 + 2] = b;
            buffer[(y * WIDTH + x + 1) * 4 + 3] = 255;

            let (r, g, b) = compute_colour(n[2]);
            buffer[(y * WIDTH + x + 2) * 4] = r;
            buffer[(y * WIDTH + x + 2) * 4 + 1] = g;
            buffer[(y * WIDTH + x + 2) * 4 + 2] = b;
            buffer[(y * WIDTH + x + 2) * 4 + 3] = 255;

            let (r, g, b) = compute_colour(n[3]);
            buffer[(y * WIDTH + x + 3) * 4] = r;
            buffer[(y * WIDTH + x + 3) * 4 + 1] = g;
            buffer[(y * WIDTH + x + 3) * 4 + 2] = b;
            buffer[(y * WIDTH + x + 3) * 4 + 3] = 255;
        }
    }
}
