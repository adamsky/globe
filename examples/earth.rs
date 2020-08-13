#![allow(warnings)]

extern crate crossterm;

use std::io::{stdout, BufRead, BufReader, Read, Write};
use std::time::Duration;

use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode, KeyEvent},
    execute, queue,
    style::Print,
    ExecutableCommand, QueueableCommand,
};

use globe::{Camera, Canvas, Globe, GlobeConfig};

fn main() {
    crossterm::terminal::enable_raw_mode().unwrap();

    let mut stdout = stdout();
    stdout.execute(cursor::Hide);

    let mut globe = GlobeConfig::new().load_texture_from("earth.txt").build();
    let mut canvas = Canvas::new(250, 250, None);

    let mut angle_offset = 0.;
    let mut cam_zoom = 2.;
    let mut cam_xy = 0.;
    let mut cam_z = 0.;
    globe.camera = Camera::new(cam_zoom, cam_xy, cam_z);
    loop {
        if poll(Duration::from_millis(100)).unwrap() {
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
                //Event::Mouse(event) => println!("{:?}", event),
                //Event::Resize(width, height) => println!("New size {}x{}", width, height),
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
        println!(
            "camera.x: {}, camera.y: {}, camera.z: {}",
            globe.camera.x, globe.camera.y, globe.camera.z
        );
        println!(
            "cam_xy: {}, cam_z: {}, cam_zoom: {}, angle: {}",
            cam_xy, cam_z, cam_zoom, globe.angle
        );
        gotoxy(0, 0);

        //update camera position
        std::thread::sleep(std::time::Duration::from_millis(10));
        //globe.angle += 1. * globe::PI / 10.;
        //angle_offset += 0. * PI / 10.;
    }
}

fn gotoxy(x: u16, y: u16) {
    //unimplemented!()
    stdout().execute(crossterm::cursor::MoveTo(x, y));
    //COORD coord = {x, y};
    //SetConsoleCursorPosition ( GetStdHandle ( STD_OUTPUT_HANDLE ), coord );
}
