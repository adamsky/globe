//! Customizable ASCII globe generator.
//!
//! Based on [C++ code by DinoZ1729](https://github.com/DinoZ1729/Earth).

#![allow(warnings)]

use std::fs::File;
use std::io::{stdout, BufRead, BufReader, Read, Write};

pub type Int = i64;
pub type Texture = Vec<Vec<char>>;

pub const PI: f64 = 3.14159265358979323846;

const palette: [char; 18] = [
    ' ', '.', ':', ';', '\'', ',', 'w', 'i', 'o', 'g', 'O', 'L', 'X', 'H', 'W', 'Y', 'V', '@',
];

pub struct Canvas {
    pub matrix: Vec<Vec<char>>,
    size: (usize, usize),
    char_pix: (usize, usize),
}
impl Canvas {
    pub fn new(x: u16, y: u16, cp: Option<(usize, usize)>) -> Self {
        let mut matrix = Vec::new();
        for i in 0..y {
            let mut row = Vec::new();
            for j in 0..x {
                row.push(' ');
            }
            matrix.push(row);
        }
        Self {
            size: (x as usize, y as usize),
            matrix,
            char_pix: cp.unwrap_or((4, 8)),
        }
    }
    pub fn get_size(&self) -> (usize, usize) {
        self.size
    }
    pub fn clear(&mut self) {
        for mut i in &mut self.matrix {
            for mut j in i {
                *j = ' ';
            }
        }
    }
    // TODO return result?
    fn draw_point(&mut self, a: usize, b: usize, c: char) {
        if a < 0 || b < 0 || a >= self.size.0 || b >= self.size.1 {
            return;
        }
        self.matrix[b][a] = c;
    }
}

pub struct Globe {
    pub camera: Camera,
    pub radius: f64,
    pub angle: f64,
    pub texture: Texture,
    pub texture_night: Option<Texture>,
}

impl Globe {
    pub fn render_on(&self, mut canvas: &mut Canvas) {
        //Sun
        let light: [f64; 3] = [0., 999999., 0.];
        //shoot the ray through every pixel

        let (sizex, sizey) = canvas.get_size();
        for yi in 0..sizey {
            let yif = yi as Int;
            for xi in 0..sizex {
                let xif = xi as Int;
                //coordinates of the camera, origin of the ray
                let o: [f64; 3] = [self.camera.x, self.camera.y, self.camera.z];
                //u is unit vector, direction of the ray
                let mut u: [f64; 3] = [
                    -((xif - (sizex / canvas.char_pix.0 / 2) as Int) as f64 + 0.5)
                        / (sizex / canvas.char_pix.0 / 2) as f64,
                    ((yif - (sizey / canvas.char_pix.1 / 2) as Int) as f64 + 0.5)
                        / (sizey / canvas.char_pix.1 / 2) as f64,
                    -1.,
                ];
                transformVector(&mut u, self.camera.matrix);
                u[0] -= self.camera.x;
                u[1] -= self.camera.y;
                u[2] -= self.camera.z;
                normalize(&mut u);
                let discriminant: f64 =
                    dot(&u, &o) * dot(&u, &o) - dot(&o, &o) + self.radius * self.radius;

                //ray doesn't hit the sphere
                if discriminant < 0. {
                    continue;
                }

                let distance: f64 = -discriminant.sqrt() - dot(&u, &o);

                //intersection point
                let inter: [f64; 3] = [
                    o[0] + distance * u[0],
                    o[1] + distance * u[1],
                    o[2] + distance * u[2],
                ];

                //surface normal
                let mut n: [f64; 3] = [
                    o[0] + distance * u[0],
                    o[1] + distance * u[1],
                    o[2] + distance * u[2],
                ];
                normalize(&mut n);
                //unit vector pointing from intersection to light source
                let mut l: [f64; 3] = [0.; 3];
                vector(&mut l, &inter, &light);
                normalize(&mut l);
                let luminance: f64 = clamp(5. * (dot(&n, &l)) + 0.5, 0., 1.);
                let mut temp: [f64; 3] = [inter[0], inter[1], inter[2]];
                rotateX(&mut temp, -PI * 2. * 0. / 360.);
                //computing coordinates for the sphere
                let phi: f64 = -temp[2] / self.radius / 2. + 0.5;
                //let t: f64 = (temp[1]/temp[0];
                let mut theta: f64 = (temp[1] / temp[0]).atan() / PI + 0.5 + self.angle / 2. / PI;
                theta -= theta.floor();
                let earthX: usize = (theta * 202.) as usize;
                let earthY: usize = (phi * 80.) as usize;
                let day = findIndex(self.texture[earthY][earthX], &palette);

                // TODO night
                //let night = findIndex(self.texture_night[earthY][earthX], &palette);
                //let index = ((1.0 - luminance) * night as f64 + luminance * day as f64) as usize;

                let index = ((1.0 - luminance) * day as f64 + luminance * day as f64) as usize;
                canvas.draw_point(xi, yi, palette[index]);
            }
        }
    }
}

#[derive(Default)]
pub struct GlobeConfig {
    camera_cfg: Option<CameraConfig>,
    radius: Option<f64>,
    angle: Option<f64>,
    template: Option<GlobeTemplate>,
    texture: Option<Texture>,
    texture_night: Option<Texture>,
}

impl GlobeConfig {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_camera(mut self, config: CameraConfig) -> Self {
        self
    }
    pub fn with_radius(mut self, r: f64) -> Self {
        self.radius = Some(r);
        self
    }
    pub fn use_template(mut self, t: GlobeTemplate) -> Self {
        self.template = Some(t);
        self
    }
    pub fn load_texture_str(mut self, texture: &str) -> Self {
        let mut tex = Texture::new();
        // let lines = BufReader::new(texture.to_string()).lines();
        let lines= texture.lines();
        for (i, line) in lines.enumerate() {
            // if let Ok(l) = line {
                let mut row = Vec::new();
                //if l.len() != 202 {
                //panic!("wrong line len");
                //}
                for (j, c) in line.chars().rev().enumerate() {
                    //print!("{}", c);
                    row.push(c);
                    //earth[i][j] = c;
                }
                tex.push(row);
            // }
        }
        self.texture = Some(tex);
        self
    }
    pub fn build(mut self) -> Globe {
        let camera = self.camera_cfg.unwrap_or(CameraConfig::default()).build();
        // TODO
        let texture = self.texture.expect("texture not provided");
        Globe {
            camera,
            radius: self.radius.unwrap_or(1.),
            angle: self.angle.unwrap_or(0.),
            texture,
            texture_night: self.texture_night,
        }
    }
}

pub enum GlobeTemplate {
    Earth,
}

pub struct CameraConfig {
    radius: f64,
    alpha: f64,
    beta: f64,
}
impl CameraConfig {
    pub fn new(radius: f64, alpha: f64, beta: f64) -> Self {
        Self {
            radius,
            alpha,
            beta,
        }
    }
    pub fn default() -> Self {
        Self {
            radius: 1.,
            alpha: 0.,
            beta: 0.,
        }
    }
    pub fn build(&self) -> Camera {
        Camera::new(self.radius, self.alpha, self.beta)
    }
}

fn findIndex(c: char, s: &[char]) -> Int {
    for i in 0..s.len() {
        if c == s[i] {
            return i as Int;
        }
    }
    return -1;
}

pub struct Camera {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    matrix: [f64; 16],
    inv: [f64; 16],
}

impl Camera {
    // alfa is camera's angle along the xy plane.
    // beta is camera's angle along z axis
    // r is the distance from the camera to the origin
    pub fn new(r: f64, alfa: f64, beta: f64) -> Self {
        let a = alfa.sin();
        let b = alfa.cos();
        let c = beta.sin();
        let d = beta.cos();

        let x = r * b * d;
        let y = r * a * d;
        let z = r * c;

        let mut matrix = [0.; 16];

        //matrix
        matrix[3] = 0.;
        matrix[7] = 0.;
        matrix[11] = 0.;
        matrix[15] = 1.;
        //x
        matrix[0] = -a;
        matrix[1] = b;
        matrix[2] = 0.;
        //y
        matrix[4] = b * c;
        matrix[5] = a * c;
        matrix[6] = -d;
        //z
        matrix[8] = b * d;
        matrix[9] = a * d;
        matrix[10] = c;

        matrix[12] = x;
        matrix[13] = y;
        matrix[14] = z;

        let mut inv = [0.; 16];

        //invert
        invert(&mut inv, matrix);

        Camera {
            x,
            y,
            z,
            matrix,
            inv,
        }
    }
}

fn transformVector(mut vec: &mut [f64; 3], m: [f64; 16]) {
    let tx: f64 = vec[0] * m[0] + vec[1] * m[4] + vec[2] * m[8] + m[12];
    let ty: f64 = vec[0] * m[1] + vec[1] * m[5] + vec[2] * m[9] + m[13];
    let tz: f64 = vec[0] * m[2] + vec[1] * m[6] + vec[2] * m[10] + m[14];
    vec[0] = tx;
    vec[1] = ty;
    vec[2] = tz;
}

fn invert(mut inv: &mut [f64; 16], matrix: [f64; 16]) {
    inv[0] = matrix[5] * matrix[10] * matrix[15]
        - matrix[5] * matrix[11] * matrix[14]
        - matrix[9] * matrix[6] * matrix[15]
        + matrix[9] * matrix[7] * matrix[14]
        + matrix[13] * matrix[6] * matrix[11]
        - matrix[13] * matrix[7] * matrix[10];

    inv[4] = -matrix[4] * matrix[10] * matrix[15]
        + matrix[4] * matrix[11] * matrix[14]
        + matrix[8] * matrix[6] * matrix[15]
        - matrix[8] * matrix[7] * matrix[14]
        - matrix[12] * matrix[6] * matrix[11]
        + matrix[12] * matrix[7] * matrix[10];

    inv[8] = matrix[4] * matrix[9] * matrix[15]
        - matrix[4] * matrix[11] * matrix[13]
        - matrix[8] * matrix[5] * matrix[15]
        + matrix[8] * matrix[7] * matrix[13]
        + matrix[12] * matrix[5] * matrix[11]
        - matrix[12] * matrix[7] * matrix[9];

    inv[12] = -matrix[4] * matrix[9] * matrix[14]
        + matrix[4] * matrix[10] * matrix[13]
        + matrix[8] * matrix[5] * matrix[14]
        - matrix[8] * matrix[6] * matrix[13]
        - matrix[12] * matrix[5] * matrix[10]
        + matrix[12] * matrix[6] * matrix[9];

    inv[1] = -matrix[1] * matrix[10] * matrix[15]
        + matrix[1] * matrix[11] * matrix[14]
        + matrix[9] * matrix[2] * matrix[15]
        - matrix[9] * matrix[3] * matrix[14]
        - matrix[13] * matrix[2] * matrix[11]
        + matrix[13] * matrix[3] * matrix[10];

    inv[5] = matrix[0] * matrix[10] * matrix[15]
        - matrix[0] * matrix[11] * matrix[14]
        - matrix[8] * matrix[2] * matrix[15]
        + matrix[8] * matrix[3] * matrix[14]
        + matrix[12] * matrix[2] * matrix[11]
        - matrix[12] * matrix[3] * matrix[10];

    inv[9] = -matrix[0] * matrix[9] * matrix[15]
        + matrix[0] * matrix[11] * matrix[13]
        + matrix[8] * matrix[1] * matrix[15]
        - matrix[8] * matrix[3] * matrix[13]
        - matrix[12] * matrix[1] * matrix[11]
        + matrix[12] * matrix[3] * matrix[9];

    inv[13] = matrix[0] * matrix[9] * matrix[14]
        - matrix[0] * matrix[10] * matrix[13]
        - matrix[8] * matrix[1] * matrix[14]
        + matrix[8] * matrix[2] * matrix[13]
        + matrix[12] * matrix[1] * matrix[10]
        - matrix[12] * matrix[2] * matrix[9];

    inv[2] = matrix[1] * matrix[6] * matrix[15]
        - matrix[1] * matrix[7] * matrix[14]
        - matrix[5] * matrix[2] * matrix[15]
        + matrix[5] * matrix[3] * matrix[14]
        + matrix[13] * matrix[2] * matrix[7]
        - matrix[13] * matrix[3] * matrix[6];

    inv[6] = -matrix[0] * matrix[6] * matrix[15]
        + matrix[0] * matrix[7] * matrix[14]
        + matrix[4] * matrix[2] * matrix[15]
        - matrix[4] * matrix[3] * matrix[14]
        - matrix[12] * matrix[2] * matrix[7]
        + matrix[12] * matrix[3] * matrix[6];

    inv[10] = matrix[0] * matrix[5] * matrix[15]
        - matrix[0] * matrix[7] * matrix[13]
        - matrix[4] * matrix[1] * matrix[15]
        + matrix[4] * matrix[3] * matrix[13]
        + matrix[12] * matrix[1] * matrix[7]
        - matrix[12] * matrix[3] * matrix[5];

    inv[14] = -matrix[0] * matrix[5] * matrix[14]
        + matrix[0] * matrix[6] * matrix[13]
        + matrix[4] * matrix[1] * matrix[14]
        - matrix[4] * matrix[2] * matrix[13]
        - matrix[12] * matrix[1] * matrix[6]
        + matrix[12] * matrix[2] * matrix[5];

    inv[3] = -matrix[1] * matrix[6] * matrix[11]
        + matrix[1] * matrix[7] * matrix[10]
        + matrix[5] * matrix[2] * matrix[11]
        - matrix[5] * matrix[3] * matrix[10]
        - matrix[9] * matrix[2] * matrix[7]
        + matrix[9] * matrix[3] * matrix[6];

    inv[7] = matrix[0] * matrix[6] * matrix[11]
        - matrix[0] * matrix[7] * matrix[10]
        - matrix[4] * matrix[2] * matrix[11]
        + matrix[4] * matrix[3] * matrix[10]
        + matrix[8] * matrix[2] * matrix[7]
        - matrix[8] * matrix[3] * matrix[6];

    inv[11] = -matrix[0] * matrix[5] * matrix[11]
        + matrix[0] * matrix[7] * matrix[9]
        + matrix[4] * matrix[1] * matrix[11]
        - matrix[4] * matrix[3] * matrix[9]
        - matrix[8] * matrix[1] * matrix[7]
        + matrix[8] * matrix[3] * matrix[5];

    inv[15] = matrix[0] * matrix[5] * matrix[10]
        - matrix[0] * matrix[6] * matrix[9]
        - matrix[4] * matrix[1] * matrix[10]
        + matrix[4] * matrix[2] * matrix[9]
        + matrix[8] * matrix[1] * matrix[6]
        - matrix[8] * matrix[2] * matrix[5];

    let mut det: f64 =
        matrix[0] * inv[0] + matrix[1] * inv[4] + matrix[2] * inv[8] + matrix[3] * inv[12];

    det = 1.0 / det;

    for i in 0..16 {
        inv[i] *= det;
    }
}

fn cross(mut r: &mut [f64; 3], a: [f64; 3], b: [f64; 3]) {
    r[0] = a[1] * b[2] - a[2] * b[1];
    r[1] = a[2] * b[0] - a[0] * b[2];
    r[2] = a[0] * b[1] - a[1] * b[0];
}

fn magnitute(r: &[f64; 3]) -> f64 {
    let s: f64 = r[0] * r[0] + r[1] * r[1] + r[2] * r[2];
    s.sqrt()
}

fn normalize(mut r: &mut [f64; 3]) {
    let len: f64 = magnitute(r);
    r[0] /= len;
    r[1] /= len;
    r[2] /= len;
}

fn dot(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn vector(mut a: &mut [f64; 3], b: &[f64; 3], c: &[f64; 3]) {
    a[0] = b[0] - c[0];
    a[1] = b[1] - c[1];
    a[2] = b[2] - c[2];
}

fn transformVector2(mut vec: &mut [f64; 3], m: &[f64; 9]) {
    let x = m[0] * vec[0] + m[1] * vec[1] + m[2] * vec[2];
    let y = m[3] * vec[0] + m[4] * vec[1] + m[5] * vec[2];
    let z = m[6] * vec[0] + m[7] * vec[1] + m[8] * vec[2];
    vec[0] = x;
    vec[1] = y;
    vec[2] = z;
}

fn rotateX(mut vec: &mut [f64; 3], theta: f64) {
    let a = theta.sin();
    let b = theta.cos();
    let m: [f64; 9] = [1., 0., 0., 0., b, -a, 0., a, b];
    transformVector2(vec, &m);
}
fn rotateY(mut vec: &mut [f64; 3], theta: f64) {
    let a = theta.sin();
    let b = theta.cos();
    let m: [f64; 9] = [b, 0., a, 0., 1., 0., -a, 0., b];
    transformVector2(vec, &m);
}
fn rotateZ(mut vec: &mut [f64; 3], theta: f64) {
    let a = theta.sin();
    let b = theta.cos();
    let m: [f64; 9] = [b, -a, 0., a, b, 0., 0., 0., 1.];
    transformVector2(vec, &m);
}

fn clamp(mut x: f64, min: f64, max: f64) -> f64 {
    if x < min {
        x = min;
    } else if x > max {
        x = max;
    }
    x
}
