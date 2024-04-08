//! Render an ASCII globe in your terminal.

#![allow(unused_variables)]

use std::f32::consts::PI;
use std::io::{stdin, stdout, Read, Stdout, Write};
use std::time::Duration;

use clap::{App, AppSettings, Arg};
use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode},
    style::Print,
    ExecutableCommand, QueueableCommand,
};
use crossterm::{event::MouseEvent, terminal};

use crossterm::terminal::ClearType;
use globe::{CameraConfig, Canvas, GlobeConfig, GlobeTemplate};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

/// Collection of scene settings that get passed from clap to mode processing
/// functions.
struct Settings {
    /// Refresh rate in cycles per second
    refresh_rate: usize,
    /// Initial globe rotation speed
    globe_rotation_speed: f32,
    /// Initial camera rotation speed
    cam_rotation_speed: f32,
    /// Initial camera zoom
    cam_zoom: f32,
    /// Target focus speed
    focus_speed: f32,
    /// Globe night side switch
    night: bool,
    /// Initial location coordinates
    coords: (f32, f32),
}

fn main() {
    let app = App::new("globe-cli")
        .version(VERSION)
        .author(AUTHORS)
        .setting(AppSettings::ArgRequiredElseHelp)
        .about("Render an ASCII globe in your terminal.")
        .arg(
            Arg::new("interactive")
                .short('i')
                .long("interactive")
                .display_order(0)
                .help("Interactive mode (input enabled)"),
        )
        .arg(
            Arg::new("screensaver")
                .short('s')
                .long("screensaver")
                .display_order(1)
                .help("Screensaver mode (input disabled)"),
        )
        .arg(
            Arg::new("refresh_rate")
                .short('r')
                .long("refresh-rate")
                .help("Refresh rate in frames per second")
                .takes_value(true)
                .value_name("fps")
                .default_value("60"),
        )
        .arg(
            Arg::new("globe_rotation")
                .short('g')
                .long("globe-rotation")
                .help("Starting globe rotation speed")
                .takes_value(true)
                .value_name("move_per_frame")
                .default_value("0"),
        )
        .arg(
            Arg::new("cam_rotation")
                .short('c')
                .long("cam-rotation")
                .help("Starting camera rotation speed")
                .takes_value(true)
                .value_name("move_per_frame")
                .default_value("0"),
        )
        .arg(
            Arg::new("cam_zoom")
                .short('z')
                .long("cam-zoom")
                .help("Starting camera zoom")
                .takes_value(true)
                .value_name("distance")
                .default_value("1.7"),
        )
        .arg(
            Arg::new("focus_speed")
                .short('f')
                .long("focus-speed")
                .help("Target focusing animation speed")
                .takes_value(true)
                .value_name("multiplier")
                .default_value("1"),
        )
        .arg(
            Arg::new("location")
                .short('l')
                .long("location")
                .help("Starting location coordinates")
                .takes_value(true)
                .value_name("coords")
                .default_value("0.4,0.6"),
        )
        .arg(
            Arg::new("night")
                .short('n')
                .long("night")
                .help("Enable displaying the night side of the globe"),
        )
        .arg(
            Arg::new("template")
                .short('t')
                .long("template")
                .help("Display a built-in globe template")
                .takes_value(true)
                .value_name("planet")
                .default_value("earth"),
        )
        .arg(
            Arg::new("texture")
                .long("texture")
                .help("Apply custom texture from file")
                .takes_value(true)
                .value_name("path"),
        )
        .arg(
            Arg::new("texture_night")
                .long("texture-night")
                .help("Apply custom night side texture from file")
                .takes_value(true)
                .value_name("path"),
        )
        .arg(
            Arg::new("pipe")
                .short('p')
                .long("pipe")
                .help("Read coordinates from stdin and display them on the globe"),
        );
    let matches = app.get_matches();

    // parse coordinates into a tuple
    let coords = matches
        .value_of("location")
        .unwrap()
        .split(",")
        .collect::<Vec<&str>>();
    if coords.len() != 2 {
        panic!("failed parsing location coordinates")
    }
    let coords: (f32, f32) = (
        coords[0]
            .parse()
            .expect("failed parsing location coordinates (first value)"),
        coords[1]
            .parse()
            .expect("failed parsing location coordinates (second value)"),
    );

    let settings = Settings {
        refresh_rate: matches
            .value_of("refresh_rate")
            .unwrap()
            .parse()
            .expect("failed parsing refresh rate value"),
        globe_rotation_speed: matches
            .value_of("globe_rotation")
            .unwrap()
            .parse()
            .expect("failed parsing globe rotation speed value"),
        cam_rotation_speed: matches
            .value_of("cam_rotation")
            .unwrap()
            .parse()
            .expect("failed parsing cam rotation speed value"),
        cam_zoom: matches
            .value_of("cam_zoom")
            .unwrap()
            .parse()
            .expect("failed parsing cam zoom value"),
        focus_speed: matches
            .value_of("focus_speed")
            .unwrap()
            .parse()
            .expect("failed parsing focus speed value"),
        night: matches.is_present("night"),
        coords,
    };

    if matches.is_present("pipe") {
        let stdin = stdin();
        let mut stdin_string = String::new();
        stdin.lock().read_to_string(&mut stdin_string).unwrap();
        let coord_list = stdin_string.split(";").collect::<Vec<&str>>();
        start_listing(settings, coord_list)
    } else if matches.is_present("interactive") {
        start_interactive(settings);
    } else if matches.is_present("screensaver") {
        start_screensaver(settings);
    }
}

/// Listing mode goes through a list of location coordinates. Pressing any key
/// triggers stepping to the next location, or if there are no more locations,
/// exits the program.
fn start_listing(settings: Settings, coords_input: Vec<&str>) {
    terminal::enable_raw_mode().unwrap();
    let mut stdout = stdout();
    stdout.execute(cursor::Hide).unwrap();
    stdout.execute(cursor::DisableBlinking).unwrap();

    let mut term_size = terminal::size().unwrap();
    let mut canvas = if term_size.0 > term_size.1 {
        Canvas::new(term_size.1 * 8, term_size.1 * 8, None)
    } else {
        Canvas::new(term_size.0 * 4, term_size.0 * 4, None)
    };

    let mut cam_zoom = settings.cam_zoom;
    let mut cam_xy = 0.;
    let mut cam_z = 0.;

    let mut globe = GlobeConfig::new()
        .use_template(GlobeTemplate::Earth)
        .with_camera(CameraConfig::new(cam_zoom, cam_xy, cam_z))
        .display_night(settings.night)
        .build();

    let coord_list: Vec<(f32, f32)> = coords_input
        .iter()
        .map(|c| {
            let split = c.split(",").collect::<Vec<&str>>();
            if split.len() != 2 {
                panic!("failed parsing coordinates, format: \"51.23,51.23\"");
            }
            (
                split[0]
                    .trim()
                    .parse()
                    .expect("failed parsing coord as float"),
                split[1]
                    .trim()
                    .parse()
                    .expect("failed parsing coord as float"),
            )
        })
        .collect();

    // set the initial coordinates
    focus_target(settings.coords, 0., &mut cam_xy, &mut cam_z);

    let globe_rot_speed = settings.globe_rotation_speed / 1000.;
    let cam_rot_speed = settings.cam_rotation_speed / 1000.;

    let mut current_index = 0;
    let mut moving_towards_target: Option<(f32, f32)> = Some(coord_list[current_index]);

    loop {
        if poll(Duration::from_millis(1000 / settings.refresh_rate as u64)).unwrap() {
            match read().unwrap() {
                // pressing any key exists the program
                Event::Key(key) => match key.code {
                    KeyCode::Char(char) => match char {
                        'c' | 'd' => break,
                        _ => {
                            current_index += 1;
                            if current_index >= coord_list.len() {
                                break;
                            }
                            moving_towards_target = Some(coord_list[current_index]);
                        }
                    },
                    _ => {
                        current_index += 1;
                        if current_index >= coord_list.len() {
                            break;
                        }
                        moving_towards_target = Some(coord_list[current_index]);
                    }
                },
                Event::Resize(width, height) => {
                    term_size = (width, height);
                    canvas = if width > height {
                        Canvas::new(height * 8, height * 8, None)
                    } else {
                        Canvas::new(width * 4, width * 4, None)
                    };
                }
                Event::Mouse(_) => (),
            }
        }

        // apply globe rotation
        globe.angle += globe_rot_speed;
        cam_xy -= globe_rot_speed / 2.;

        // apply camera rotation
        cam_xy -= cam_rot_speed;

        if let Some(target_coords) = moving_towards_target {
            if move_towards_target(
                settings.focus_speed,
                target_coords,
                cam_zoom,
                globe.angle / 2.,
                &mut cam_xy,
                &mut cam_z,
                &mut cam_zoom,
            ) {
                moving_towards_target = None;
            }
        }

        globe.camera.update(cam_zoom, cam_xy, cam_z);

        // render globe on the canvas
        canvas.clear();
        globe.render_on(&mut canvas);

        // print canvas to terminal
        print_canvas(&mut canvas, &term_size, &mut stdout);
    }

    stdout.execute(cursor::Show).unwrap();
    stdout.execute(cursor::EnableBlinking).unwrap();

    terminal::disable_raw_mode().unwrap();
    stdout.execute(terminal::Clear(ClearType::All)).unwrap();
}

/// Screensaver mode doesn't allow for user input. Any key press exits the
/// program.
fn start_screensaver(settings: Settings) {
    terminal::enable_raw_mode().unwrap();
    let mut stdout = stdout();
    stdout.execute(cursor::Hide).unwrap();
    stdout.execute(cursor::DisableBlinking).unwrap();

    let mut term_size = terminal::size().unwrap();
    let mut canvas = if term_size.0 > term_size.1 {
        Canvas::new(term_size.1 * 8, term_size.1 * 8, None)
    } else {
        Canvas::new(term_size.0 * 4, term_size.0 * 4, None)
    };

    let cam_zoom = settings.cam_zoom;
    let mut cam_xy = 0.;
    let mut cam_z = 0.;

    // set the initial coordinates
    focus_target(settings.coords, 0., &mut cam_xy, &mut cam_z);

    let mut globe = GlobeConfig::new()
        .use_template(GlobeTemplate::Earth)
        .with_camera(CameraConfig::new(cam_zoom, cam_xy, cam_z))
        .display_night(settings.night)
        .build();

    let globe_rot_speed = settings.globe_rotation_speed / 1000.;
    let cam_rot_speed = settings.cam_rotation_speed / 1000.;

    loop {
        if poll(Duration::from_millis(1000 / settings.refresh_rate as u64)).unwrap() {
            match read().unwrap() {
                // pressing any key exists the program
                Event::Key(_) => break,
                Event::Resize(width, height) => {
                    term_size = (width, height);
                    canvas = if width > height {
                        Canvas::new(height * 8, height * 8, None)
                    } else {
                        Canvas::new(width * 4, width * 4, None)
                    };
                }
                Event::Mouse(_) => (),
            }
        }

        // apply globe rotation
        globe.angle += globe_rot_speed;
        cam_xy -= globe_rot_speed / 2.;

        // apply camera rotation
        cam_xy -= cam_rot_speed;

        globe.camera.update(cam_zoom, cam_xy, cam_z);

        // render globe on the canvas
        canvas.clear();
        globe.render_on(&mut canvas);

        // print canvas to terminal
        print_canvas(&mut canvas, &term_size, &mut stdout);
    }

    stdout.execute(cursor::Show).unwrap();
    stdout.execute(cursor::EnableBlinking).unwrap();

    terminal::disable_raw_mode().unwrap();
    stdout.execute(terminal::Clear(ClearType::All)).unwrap();
}

/// Interactive mode allows using mouse and/or keyboard to control the globe.
fn start_interactive(settings: Settings) {
    terminal::enable_raw_mode().unwrap();
    let mut stdout = stdout();
    stdout.execute(cursor::Hide).unwrap();
    stdout.execute(cursor::DisableBlinking).unwrap();
    stdout
        .execute(crossterm::event::EnableMouseCapture)
        .unwrap();

    let mut term_size = terminal::size().unwrap();
    let mut canvas = if term_size.0 > term_size.1 {
        Canvas::new(term_size.1 * 8, term_size.1 * 8, None)
    } else {
        Canvas::new(term_size.0 * 4, term_size.0 * 4, None)
    };

    let mut cam_zoom = settings.cam_zoom;
    let mut cam_xy = 0.;
    let mut cam_z = 0.;

    // set the initial coordinates
    focus_target(settings.coords, 0., &mut cam_xy, &mut cam_z);

    let mut globe = GlobeConfig::new()
        .use_template(GlobeTemplate::Earth)
        .with_camera(CameraConfig::new(cam_zoom, cam_xy, cam_z))
        .display_night(settings.night)
        .build();

    let mut globe_rot_speed = settings.globe_rotation_speed / 1000.;
    let mut cam_rot_speed = settings.cam_rotation_speed / 1000.;

    let mut last_drag_pos = None;
    let mut moving_towards_target: Option<(f32, f32)> = None;

    loop {
        if poll(Duration::from_millis(1000 / settings.refresh_rate as u64)).unwrap() {
            match read().unwrap() {
                Event::Key(event) => match event.code {
                    KeyCode::Char(char) => match char {
                        '-' => globe_rot_speed -= 0.005,
                        '+' => globe_rot_speed += 0.005,
                        ',' => cam_rot_speed -= 0.005,
                        '.' => cam_rot_speed += 0.005,
                        'n' => globe.display_night = !globe.display_night,
                        // vim-style navigation with hjkl
                        'h' => cam_xy += 0.1,
                        'l' => cam_xy -= 0.1,
                        'k' => {
                            if cam_z < 1.5 {
                                cam_z += 0.1;
                            }
                        }
                        'j' => {
                            if cam_z > -1.5 {
                                cam_z -= 0.1;
                            }
                        }
                        _ => break,
                    },
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
                    KeyCode::Left => cam_xy += 0.1,
                    KeyCode::Right => cam_xy -= 0.1,
                    KeyCode::Enter => {
                        focus_target(settings.coords, globe.angle / 2., &mut cam_xy, &mut cam_z);
                        // moving_towards_target = Some(settings.coords);
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

                            cam_xy += x_diff * PI / 30.;
                            cam_xy += y_diff * PI / 30.;
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

        // apply globe rotation
        globe.angle += globe_rot_speed;
        cam_xy -= globe_rot_speed / 2.;

        // apply camera rotation
        cam_xy -= cam_rot_speed;

        // clip camera zoom
        if cam_zoom < 1.0 {
            cam_zoom = 1.0;
        }

        if let Some(target_coords) = moving_towards_target {
            if move_towards_target(
                settings.focus_speed,
                target_coords,
                cam_zoom,
                globe.angle / 2.,
                &mut cam_xy,
                &mut cam_z,
                &mut cam_zoom,
            ) {
                moving_towards_target = None;
            }
        }

        globe.camera.update(cam_zoom, cam_xy, cam_z);

        // render globe on the canvas
        canvas.clear();
        globe.render_on(&mut canvas);

        // print canvas to terminal
        print_canvas(&mut canvas, &term_size, &mut stdout);
    }

    stdout.execute(cursor::Show).unwrap();
    stdout.execute(cursor::EnableBlinking).unwrap();
    stdout
        .execute(crossterm::event::DisableMouseCapture)
        .unwrap();

    terminal::disable_raw_mode().unwrap();
    stdout.execute(terminal::Clear(ClearType::All)).unwrap();
}

/// Prints globe canvas to stdout.
fn print_canvas(canvas: &mut Canvas, term_size: &(u16, u16), stdout: &mut Stdout) {
    let (canvas_size_x, canvas_size_y) = canvas.get_size();
    for i in 0..canvas_size_y / canvas.char_pix.1 {
        stdout
            .queue(terminal::Clear(terminal::ClearType::CurrentLine))
            .unwrap();
        for j in 0..canvas_size_x / canvas.char_pix.0 {
            stdout.queue(Print(canvas.matrix[i][j])).unwrap();
        }
        stdout.queue(cursor::MoveDown(1)).unwrap();
        stdout
            .queue(cursor::MoveLeft((canvas_size_x / 4) as u16))
            .unwrap();
        stdout.flush().unwrap();
    }

    if term_size.0 / 2 > term_size.1 {
        stdout
            .execute(crossterm::cursor::MoveTo(
                (canvas_size_x / canvas.char_pix.1) as u16
                    - ((canvas_size_x / canvas.char_pix.1) / canvas.char_pix.0) as u16,
                0,
            ))
            .unwrap();
    }
}

/// Orients the camera so that it focuses on the given target coordinates.
pub fn focus_target(coords: (f32, f32), xy_offset: f32, cam_xy: &mut f32, cam_z: &mut f32) {
    let (cx, cy) = coords;
    *cam_xy = (cx * PI) * -1. - 1.5 - xy_offset;
    *cam_z = cy * 3. - 1.5;
}

//TODO animate zoom
/// Rotates the camera towards given target coordinates.
pub fn move_towards_target(
    speed: f32,
    coords: (f32, f32),
    target_zoom: f32,
    xy_offset: f32,
    cam_xy: &mut f32,
    cam_z: &mut f32,
    cam_zoom: &mut f32,
) -> bool {
    let (cx, cy) = coords;
    let target_xy = (cx * PI - xy_offset) * -1. - 1.5;
    let target_z = cy * 3. - 1.5;

    let diff_xy = target_xy - *cam_xy;
    let diff_z = target_z - *cam_z;

    if diff_xy.abs() < 0.01 && diff_z.abs() < 0.01 {
        return true;
    }

    let mut xy_move = 0.01 * speed + (diff_xy.abs() / 30. * speed);
    if diff_xy.abs() < 0.07 {
        xy_move = xy_move / 5.;
    }
    if diff_xy > 0. {
        *cam_xy += xy_move;
    } else if diff_xy < 0. {
        *cam_xy -= xy_move;
    }

    let mut z_move = 0.005 * speed + (diff_z.abs() / 30. * speed);
    if diff_z.abs() < 0.07 {
        z_move = z_move / 5.;
    }
    if diff_z > 0. {
        *cam_z += z_move;
    } else if diff_z < 0. {
        *cam_z -= z_move;
    }

    false
}
