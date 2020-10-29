//! Render an ASCII globe in your terminal.
//!
//! # Interactive mode controls
//!
//! **Mouse**: click and drag to rotate the globe. Use the mouse wheel to zoom in
//! and out.
//!
//! **Keyboard**: use arrow keys to rotate, *PgUp* and *PgDown* to zoom.

use std::f32::consts::PI;
use std::io::{stdout, Write};
use std::time::Duration;

use clap::{App, AppSettings, Arg};
use crossterm::{event::MouseEvent, terminal};
use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode},
    style::Print,
    ExecutableCommand, QueueableCommand,
};

use globe::{Camera, Canvas, GlobeConfig, GlobeTemplate};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

fn main() {
    let app = App::new("globe-cli")
        .version(VERSION)
        .author(AUTHORS)
        .setting(AppSettings::ArgRequiredElseHelp)
        .about("Render an ASCII globe in your terminal.")
        .arg(Arg::new("interactive").short('i'))
        .arg(Arg::new("screensaver").short('s'));
    let matches = app.get_matches();
    if matches.is_present("interactive") {
        start_interactive();
    } else if matches.is_present("screensaver") {
        start_screensaver();
    }
}

fn start_screensaver() {
    terminal::enable_raw_mode().unwrap();

    let mut stdout = stdout();
    stdout.execute(cursor::Hide).unwrap();
    stdout.execute(cursor::DisableBlinking).unwrap();

    let mut globe = GlobeConfig::new()
        .use_template(GlobeTemplate::Earth)
        .build();
    let mut term_size = terminal::size().unwrap();
    let mut canvas = if term_size.0 > term_size.1 {
        Canvas::new(term_size.1 * 8, term_size.1 * 8, None)
    } else {
        Canvas::new(term_size.0 * 4, term_size.0 * 4, None)
    };

    let cam_zoom = 2.;
    let cam_xy = 0.;
    let cam_z = 0.;
    globe.camera = Camera::new(cam_zoom, cam_xy, cam_z);

    loop {
        if poll(Duration::from_millis(100)).unwrap() {
            match read().unwrap() {
                Event::Key(event) => {
                    // pressing any char key exists the program
                    if let KeyCode::Char(_) = event.code {
                        break
                    }
                }
                Event::Resize(width, height) => {
                    term_size = (width, height);
                    canvas = if width > height {
                        Canvas::new(height * 8, height * 8, None)
                    } else {
                        Canvas::new(width * 4, width * 4, None)
                    };
                }
                Event::Mouse(_) => {}
            }
        }

        // make the globe spin
        globe.angle -= PI / 50.;

        globe.camera = Camera::new(cam_zoom, cam_xy, cam_z);
        canvas.clear();

        // render globe on the canvas
        globe.render_on(&mut canvas);

        // print canvas to terminal
        let (size_x, size_y) = canvas.get_size();
        for i in 0..size_y / 8 {
            stdout.queue(terminal::Clear(
                terminal::ClearType::CurrentLine,
            )).unwrap();
            for j in 0..size_x / 4 {
                stdout.queue(Print(canvas.matrix[i][j])).unwrap();
            }
            stdout.queue(cursor::MoveDown(1)).unwrap();
            stdout.queue(cursor::MoveLeft((size_x / 4) as u16)).unwrap();
            stdout.flush().unwrap();
        }

        if term_size.0 / 2 > term_size.1 {
            // center the cursor on the x axis
            stdout.execute(crossterm::cursor::MoveTo(
                (size_x / 8) as u16 - ((size_x / 8) / 4) as u16,
                0,
            )).unwrap();
        }
    }

    stdout.execute(cursor::Show).unwrap();
    stdout.execute(cursor::EnableBlinking).unwrap();

    terminal::disable_raw_mode().unwrap();
}

fn start_interactive() {
    terminal::enable_raw_mode().unwrap();

    let mut stdout = stdout();
    stdout.execute(cursor::Hide).unwrap();
    stdout.execute(cursor::DisableBlinking).unwrap();
    stdout.execute(crossterm::event::EnableMouseCapture).unwrap();

    let mut globe = GlobeConfig::new()
        .use_template(GlobeTemplate::Earth)
        .build();
    let mut term_size = terminal::size().unwrap();
    let mut canvas = if term_size.0 > term_size.1 {
        Canvas::new(term_size.1 * 8, term_size.1 * 8, None)
    } else {
        Canvas::new(term_size.0 * 4, term_size.0 * 4, None)
    };

    let mut cam_zoom = 2.;
    let cam_xy = 0.;
    let mut cam_z = 0.;
    globe.camera = Camera::new(cam_zoom, cam_xy, cam_z);

    let mut last_drag_pos = None;

    loop {
        if poll(Duration::from_millis(100)).unwrap() {
            match read().unwrap() {
                Event::Key(event) => match event.code {
                    KeyCode::Char(_) => break,
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
                    KeyCode::Left => globe.angle += PI / 30.,
                    KeyCode::Right => globe.angle -= PI / 30.,
                    KeyCode::Enter => {
                        // focus on point
                        let (cx, cy) = (0., 0.);

                        cam_z = cy * 3. - 1.5;
                        globe.angle = cx * (PI * 2.) + PI;
                    }
                    _ => (),
                },
                Event::Mouse(event) => match event {
                    MouseEvent::Drag(_, x, y, _) => {
                        if let Some(last) = last_drag_pos {
                            let (x_last, y_last) = last;
                            let x_diff = x as globe::Float - x_last as globe::Float;
                            let y_diff = y as globe::Float - y_last as globe::Float;

                            if y_diff > 0. && cam_z < 1.5 {
                                cam_z += 0.1;
                            } else if y_diff < 0. && cam_z > -1.5 {
                                cam_z -= 0.1;
                            }
                            globe.angle += x_diff * PI / 30.;
                            globe.angle += y_diff * PI / 30.;
                        }
                        last_drag_pos = Some((x, y))
                    }
                    MouseEvent::ScrollUp(..) => cam_zoom -= 0.1,
                    MouseEvent::ScrollDown(..) => cam_zoom += 0.1,
                    _ => last_drag_pos = None,
                },
                Event::Resize(width, height) => {
                    term_size = (width, height);
                    canvas = if width > height {
                        Canvas::new(height * 8, height * 8, None)
                    } else {
                        Canvas::new(width * 4, width * 4, None)
                    };
                }
            }
        }

        // clip camera zoom
        if cam_zoom < 1.0 {
            cam_zoom = 1.0;
        }

        globe.camera = Camera::new(cam_zoom, cam_xy, cam_z);

        canvas.clear();

        // render globe on the canvas
        globe.render_on(&mut canvas);

        // print canvas to terminal
        let (size_x, size_y) = canvas.get_size();
        for i in 0..size_y / 8 {
            stdout.queue(terminal::Clear(
                terminal::ClearType::CurrentLine,
            )).unwrap();
            for j in 0..size_x / 4 {
                stdout.queue(Print(canvas.matrix[i][j])).unwrap();
            }
            // stdout.execute(cursor::MoveToNextLine(1));
            stdout.queue(cursor::MoveDown(1)).unwrap();
            stdout.queue(cursor::MoveLeft((size_x / 4) as u16)).unwrap();
            stdout.flush().unwrap();
        }

        if term_size.0 / 2 > term_size.1 {
            // center the cursor on the x axis
            stdout.execute(crossterm::cursor::MoveTo(
                (size_x / 8) as u16 - ((size_x / 8) / 4) as u16,
                // (term_size.0 / 2) - (term_size.0 / 4) as u16,
                // term_size.0 / 2,
                0,
            )).unwrap();
        }
    }

    stdout.execute(cursor::Show).unwrap();
    stdout.execute(cursor::EnableBlinking).unwrap();
    stdout.execute(crossterm::event::DisableMouseCapture).unwrap();

    terminal::disable_raw_mode().unwrap();
}
