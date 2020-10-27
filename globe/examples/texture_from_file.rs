use globe::{Camera, CameraConfig, Canvas, GlobeConfig};
use std::iter::FromIterator;

fn main() {
    // use config builder to create a new globe struct
    let mut globe = GlobeConfig::new()
        // specify path to the texture file
        .with_texture_at("textures/earth.txt")
        // for built-in textures try using a template
        //.use_template(GlobeTemplate::Earth)
        .with_camera(CameraConfig::default())
        .build();
    // create a new canvas
    let mut canvas = Canvas::new(250, 250, None);
    // render the globe onto the canvas
    globe.render_on(&mut canvas);
    // print out the canvas
    let (sizex, sizey) = canvas.get_size();
    for i in 0..sizey / 8 {
        for j in 0..sizex / 4 {
            print!("{}", canvas.matrix[i][j]);
        }
        println!();
    }
}
