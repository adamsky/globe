//! This example displays an earth globe based on the example ascii earth
//! texture.
//!
//! # Mouse controls
//!
//! Click and drag to rotate the globe. Use the mouse wheel to zoom in and out.
//!
//! # Keyboard controls
//!
//! Use arrow keys to rotate, *PgUp* and *PgDown* to zoom.

#![allow(warnings)]

extern crate crossterm;

use std::io::{stdout, BufRead, BufReader, Read, Write};
use std::path::Path;
use std::time::Duration;

use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode, KeyEvent},
    execute, queue,
    style::Print,
    ExecutableCommand, QueueableCommand,
};

use crossterm::event::MouseEvent;
use crossterm::terminal::enable_raw_mode;
use globe::{Camera, Canvas, Globe, GlobeConfig};

fn main() {
    crossterm::terminal::enable_raw_mode().unwrap();

    let mut stdout = stdout();
    stdout.execute(cursor::Hide);
    stdout.execute(crossterm::event::EnableMouseCapture);

    let mut earth_texture_path = std::env::current_dir().unwrap();
    earth_texture_path.push("examples/ascii/earth.txt");

    let mut globe = GlobeConfig::new()
        .load_texture_from(earth_texture_path.to_str().unwrap())
        .build();
    // let mut canvas = Canvas::new(450, 450, None);
    let (w, h) = crossterm::terminal::size().unwrap();
    let mut canvas = Canvas::new(h * 8, h * 8, None);

    let mut angle_offset = 0.;
    let mut cam_zoom = 2.;
    let mut cam_xy = 0.;
    let mut cam_z = 0.;
    globe.camera = Camera::new(cam_zoom, cam_xy, cam_z);

    let mut last_drag_pos = None;

    loop {
        if poll(Duration::from_millis(10)).unwrap() {
            match read().unwrap() {
                Event::Key(event) => match event.code {
                    KeyCode::Char(c) => return,
                    KeyCode::PageUp => cam_zoom += 0.1,
                    KeyCode::PageDown => cam_zoom -= 0.1,
                    KeyCode::Up => {
                        if cam_z < 1.5 {
                            cam_z += 0.1;
                        }
                    }
                    KeyCode::Down => {
                        if cam_z > -1.5 {
                            cam_z -= 0.1;
                        }
                    }
                    KeyCode::Down => cam_z -= 0.1,
                    KeyCode::Left => globe.angle += 1. * globe::PI / 30.,
                    KeyCode::Right => globe.angle += -1. * globe::PI / 30.,
                    KeyCode::Enter => {
                        // focus on point
                        //let coord = (0.6, 0.7);
                        //let coord = (0.5, 0.5);
                        let coord = (0., 0.);
                        let (cx, cy) = coord;

                        let target_cam_z = cy * 3. - 1.5;
                        cam_z = target_cam_z;

                        let target_angle = cx * (globe::PI * 2.) + globe::PI;
                        globe.angle = target_angle;
                    }
                    _ => (),
                },
                Event::Mouse(event) => match event {
                    MouseEvent::Drag(_, x, y, _) => {
                        if let Some(last) = last_drag_pos {
                            let (x_last, y_last) = last;
                            let x_diff = x as f64 - x_last as f64;
                            let y_diff = y as f64 - y_last as f64;

                            if y_diff > 0. && cam_z < 1.5 {
                                cam_z += 0.1;
                            } else if y_diff < 0. && cam_z > -1.5 {
                                cam_z -= 0.1;
                            }
                            globe.angle += x_diff * globe::PI / 30.;
                            globe.angle += y_diff * globe::PI / 30.;
                        }
                        last_drag_pos = Some((x, y))
                    }
                    MouseEvent::ScrollUp(..) => cam_zoom -= 0.1,
                    MouseEvent::ScrollDown(..) => cam_zoom += 0.1,
                    _ => last_drag_pos = None,
                },
                Event::Resize(width, height) => {
                    let ratio = width as f64 / height as f64;
                    canvas = Canvas::new((height * 6) as u16, height * 6, None);
                    // 200 / 60 = 3.3
                }
                _ => (),
            }
        }

        globe.camera = Camera::new(cam_zoom, cam_xy, cam_z);
        canvas.clear();

        // render globe on the canvas
        globe.render_on(&mut canvas);

        // print canvas to terminal
        let (sizex, sizey) = canvas.get_size();
        for i in 0..sizey / 8 {
            for j in 0..sizex / 4 {
                stdout.execute(Print(canvas.matrix[i][j]));
            }
            stdout.execute(cursor::MoveToNextLine(1));
        }

        stdout.execute(crossterm::terminal::Clear(
            crossterm::terminal::ClearType::FromCursorDown,
        ));

        stdout.execute(crossterm::cursor::MoveTo(0, 0));

        //update camera position
        // std::thread::sleep(std::time::Duration::from_millis(10));
        //globe.angle += 1. * globe::PI / 10.;
        //angle_offset += 0. * PI / 10.;
    }
}
