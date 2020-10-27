![](.github/globe_logo.png)

Render an ASCII globe in your terminal. Make it interactive or just let it
spin in the background.

Currently only able to display single earth texture.

![](.github/earth_dragging.gif)

## Install

Use `cargo install`:
```
cargo install globe-cli
```

Or `git clone` and `cargo run --release` yourself.


## Run

Display a globe in *screensaver* mode using the `-s` option. Optionally pass
it the target speed of rotation as an argument: 
```
globe -s 
```

Alternatively start an interactive mode, where you can pan the globe around,
using either the mouse or keyboard arrows:
```
globe -i
```


## Use the library

To use `globe` within your Rust project, add it to your dependencies:
```
[dependencies]
globe = "0.1.0"
```

First create a `Globe`:
```
let mut globe = GlobeConfig::new()
    .use_template(GlobeTemplate::Earth)
    .with_camera(CameraConfig::default())
    .build();
```

Next make a new `Canvas` and render the `Globe` onto it:
```
let mut canvas = Canvas::new(250, 250, None);
globe.render_on(&mut canvas);
```

You can now print out the canvas to the terminal:
```
let (sizex, sizey) = canvas.get_size();
// default character size is 4 by 8
for i in 0..sizey / 8 {
    for j in 0..sizex / 4 {
        print!("{}", canvas.matrix[i][j]);
    }
    println!();
}
``` 

See `globe-cli` code for examples of runtime changes to the `Globe` and it's
`Camera`.

## Credits

Rendering math based on 
[C++ code by DinoZ1729](https://github.com/DinoZ1729/Earth).