//! Customizable ASCII globe generator.
//!
//! Based on [C++ code by DinoZ1729](https://github.com/DinoZ1729/Earth).

#![allow(warnings)]

use std::fs::File;
use std::io::{stdout, BufRead, BufReader, Read, Write};

pub type Int = i32;
pub type Float = f32;
pub type Texture = Vec<Vec<char>>;

pub const PI: Float = std::f32::consts::PI;

const palette: [char; 15] = [
    ' ', '`', '.', '-', ':', '/', '+', 'o', 's', 'y', 'h', 'd', 'm', 'N', 'M',
];

static EARTH_TEXTURE: &'static str = include_str!("../textures/earth.txt");

pub struct Canvas {
    pub matrix: Vec<Vec<char>>,
    size: (usize, usize),
    // character size
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
    fn draw_point(&mut self, a: usize, b: usize, c: char) {
        if a < 0 || b < 0 || a >= self.size.0 || b >= self.size.1 {
            return;
        }
        self.matrix[b][a] = c;
    }
}

pub struct Globe {
    pub camera: Camera,
    pub radius: Float,
    pub angle: Float,
    pub texture: Texture,
    pub texture_night: Option<Texture>,
}

impl Globe {
    pub fn render_on(&self, mut canvas: &mut Canvas) {
        //Sun
        let light: [Float; 3] = [0., 999999., 0.];
        //shoot the ray through every pixel

        let (sizex, sizey) = canvas.get_size();
        let textureWidth = (self.texture[0].len() - 1) as f32;
        let textureHeight = (self.texture.len() - 1) as f32;
        
        for yi in 0..sizey {
            let yif = yi as Int;
            for xi in 0..sizex {
                let xif = xi as Int;
                //coordinates of the camera, origin of the ray
                let o: [Float; 3] = [self.camera.x, self.camera.y, self.camera.z];
                //u is unit vector, direction of the ray
                let mut u: [Float; 3] = [
                    -((xif - (sizex / canvas.char_pix.0 / 2) as Int) as Float + 0.5)
                        / (sizex / canvas.char_pix.0 / 2) as Float,
                    ((yif - (sizey / canvas.char_pix.1 / 2) as Int) as Float + 0.5)
                        / (sizey / canvas.char_pix.1 / 2) as Float,
                    -1.,
                ];
                transformVector(&mut u, self.camera.matrix);
                u[0] -= self.camera.x;
                u[1] -= self.camera.y;
                u[2] -= self.camera.z;
                normalize(&mut u);
                let discriminant: Float =
                    dot(&u, &o) * dot(&u, &o) - dot(&o, &o) + self.radius * self.radius;

                //ray doesn't hit the sphere
                if discriminant < 0. {
                    continue;
                }

                let distance: Float = -discriminant.sqrt() - dot(&u, &o);

                //intersection point
                let inter: [Float; 3] = [
                    o[0] + distance * u[0],
                    o[1] + distance * u[1],
                    o[2] + distance * u[2],
                ];

                //surface normal
                let mut n: [Float; 3] = [
                    o[0] + distance * u[0],
                    o[1] + distance * u[1],
                    o[2] + distance * u[2],
                ];
                normalize(&mut n);
                //unit vector pointing from intersection to light source
                let mut l: [Float; 3] = [0.; 3];
                vector(&mut l, &inter, &light);
                normalize(&mut l);
                let luminance: Float = clamp(5. * (dot(&n, &l)) + 0.5, 0., 1.);
                let mut temp: [Float; 3] = [inter[0], inter[1], inter[2]];
                rotateX(&mut temp, -PI * 2. * 0. / 360.);
                //computing coordinates for the sphere
                let phi: Float = -temp[2] / self.radius / 2. + 0.5;
                //let t: Float = (temp[1]/temp[0];
                let mut theta: Float = (temp[1] / temp[0]).atan() / PI + 0.5 + self.angle / 2. / PI;
                theta -= theta.floor();
                let earthX: usize = (theta * textureWidth) as usize;
                let earthY: usize = (phi * textureHeight) as usize;
                let day = findIndex(self.texture[earthY][earthX], &palette);

                // TODO night
                //let night = findIndex(self.texture_night[earthY][earthX], &palette);
                //let index = ((1.0 - luminance) * night as Float + luminance * day as Float) as usize;

                let index = ((1.0 - luminance) * day as Float + luminance * day as Float) as usize;
                canvas.draw_point(xi, yi, palette[index]);
            }
        }
    }
}

#[derive(Default)]
pub struct GlobeConfig {
    camera_cfg: Option<CameraConfig>,
    radius: Option<Float>,
    angle: Option<Float>,
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
    pub fn with_radius(mut self, r: Float) -> Self {
        self.radius = Some(r);
        self
    }
    pub fn use_template(mut self, t: GlobeTemplate) -> Self {
        self.template = Some(t);
        self
    }
    pub fn with_texture(mut self, texture: &str) -> Self {
        let mut tex = Texture::new();
        let lines = texture.lines();
        for (i, line) in lines.enumerate() {
            let mut row = Vec::new();
            for (j, c) in line.chars().rev().enumerate() {
                row.push(c);
            }
            tex.push(row);
        }
        self.texture = Some(tex);
        self
    }
    pub fn with_texture_at(mut self, path: &str) -> Self {
        let mut file = File::open(path).unwrap();
        let mut out_string = String::new();
        file.read_to_string(&mut out_string);
        self.with_texture(&out_string)
    }
    pub fn build(mut self) -> Globe {
        if let Some(template) = &self.template {
            match template {
                GlobeTemplate::Earth => self = self.with_texture(EARTH_TEXTURE),
            }
        }
        let texture = self.texture.expect("texture not provided");
        let camera = self.camera_cfg.unwrap_or(CameraConfig::default()).build();
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
    radius: Float,
    alpha: Float,
    beta: Float,
}
impl CameraConfig {
    pub fn new(radius: Float, alpha: Float, beta: Float) -> Self {
        Self {
            radius,
            alpha,
            beta,
        }
    }
    pub fn default() -> Self {
        Self {
            radius: 2.,
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
    pub x: Float,
    pub y: Float,
    pub z: Float,
    matrix: [Float; 16],
    inv: [Float; 16],
}

impl Camera {
    // alfa is camera's angle along the xy plane.
    // beta is camera's angle along z axis
    // r is the distance from the camera to the origin
    pub fn new(r: Float, alfa: Float, beta: Float) -> Self {
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

fn transformVector(mut vec: &mut [Float; 3], m: [Float; 16]) {
    let tx: Float = vec[0] * m[0] + vec[1] * m[4] + vec[2] * m[8] + m[12];
    let ty: Float = vec[0] * m[1] + vec[1] * m[5] + vec[2] * m[9] + m[13];
    let tz: Float = vec[0] * m[2] + vec[1] * m[6] + vec[2] * m[10] + m[14];
    vec[0] = tx;
    vec[1] = ty;
    vec[2] = tz;
}

fn invert(mut inv: &mut [Float; 16], matrix: [Float; 16]) {
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

    let mut det: Float =
        matrix[0] * inv[0] + matrix[1] * inv[4] + matrix[2] * inv[8] + matrix[3] * inv[12];

    det = 1.0 / det;

    for i in 0..16 {
        inv[i] *= det;
    }
}

fn cross(mut r: &mut [Float; 3], a: [Float; 3], b: [Float; 3]) {
    r[0] = a[1] * b[2] - a[2] * b[1];
    r[1] = a[2] * b[0] - a[0] * b[2];
    r[2] = a[0] * b[1] - a[1] * b[0];
}

fn magnitute(r: &[Float; 3]) -> Float {
    let s: Float = r[0] * r[0] + r[1] * r[1] + r[2] * r[2];
    s.sqrt()
}

fn normalize(mut r: &mut [Float; 3]) {
    let len: Float = magnitute(r);
    r[0] /= len;
    r[1] /= len;
    r[2] /= len;
}

fn dot(a: &[Float; 3], b: &[Float; 3]) -> Float {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn vector(mut a: &mut [Float; 3], b: &[Float; 3], c: &[Float; 3]) {
    a[0] = b[0] - c[0];
    a[1] = b[1] - c[1];
    a[2] = b[2] - c[2];
}

fn transformVector2(mut vec: &mut [Float; 3], m: &[Float; 9]) {
    let x = m[0] * vec[0] + m[1] * vec[1] + m[2] * vec[2];
    let y = m[3] * vec[0] + m[4] * vec[1] + m[5] * vec[2];
    let z = m[6] * vec[0] + m[7] * vec[1] + m[8] * vec[2];
    vec[0] = x;
    vec[1] = y;
    vec[2] = z;
}

fn rotateX(mut vec: &mut [Float; 3], theta: Float) {
    let a = theta.sin();
    let b = theta.cos();
    let m: [Float; 9] = [1., 0., 0., 0., b, -a, 0., a, b];
    transformVector2(vec, &m);
}
fn rotateY(mut vec: &mut [Float; 3], theta: Float) {
    let a = theta.sin();
    let b = theta.cos();
    let m: [Float; 9] = [b, 0., a, 0., 1., 0., -a, 0., b];
    transformVector2(vec, &m);
}
fn rotateZ(mut vec: &mut [Float; 3], theta: Float) {
    let a = theta.sin();
    let b = theta.cos();
    let m: [Float; 9] = [b, -a, 0., a, b, 0., 0., 0., 1.];
    transformVector2(vec, &m);
}

fn clamp(mut x: Float, min: Float, max: Float) -> Float {
    if x < min {
        x = min;
    } else if x > max {
        x = max;
    }
    x
}
