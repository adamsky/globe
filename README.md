![](.github/globe_logo.png)

Render an ASCII globe in your terminal. Make it interactive or just let it
spin in the background.

![](.github/earth_dragging.gif)

## Changelog

v0.2.1:
- upgraded `clap` dependency to `3.0.0`
- changed `globe-cli` `template` argument to not be required

v0.2.0:
- added multiple CLI arguments for setting up the scene (`refresh-rate`, 
`globe-rotation`, `cam-rotation`, `cam-zoom`, `location`, `focus-speed`,
`night`, `template`, `texture`, `texture-night`)
- added experimental *listing mode* that supports reading coordinates from
standard input and going through all of them, animating camera target changes
(see `--pipe`)
- enabled ability to display night side of the globe using an additional
texture
- changed default Earth texture (now includes New Zealand)
- added vim-style navigation for the interactive mode
- improved internal library representation of `Texture`
- improved documentation

v0.1.2
- added clearing screen on exit
- fixed panic when using rust version <1.45

v0.1.1
- fixed mouse capture staying on after exit

v0.1.0
- initial release

## Install

To build `globe-cli` you will need to have 
[Rust programming language](https://rustup.rs) installed on your machine. 

Use `cargo install`:
```
cargo install globe-cli
```

Or `git clone` and `cargo run --release` directly from the repository.

### AUR

`globe` can be installed from available [AUR packages](https://aur.archlinux.org/packages/?O=0&SeB=b&K=globe-cli&outdated=&SB=n&SO=a&PP=50&do_Search=Go) using an [AUR helper](https://wiki.archlinux.org/index.php/AUR_helpers). For example,

```
yay -S globe-cli
```

If you prefer, you can clone the [AUR packages](https://aur.archlinux.org/packages/?O=0&SeB=b&K=globe-cli&outdated=&SB=n&SO=a&PP=50&do_Search=Go) and then compile them with [makepkg](https://wiki.archlinux.org/index.php/Makepkg). For example,

```
git clone https://aur.archlinux.org/globe-cli.git
cd globe-cli
makepkg -si
```

### Docker

You can also use Docker to try out `globe`, no Rust needed. After cloning the repo, just build and run an image from the `Dockerfile` contained at the root of the project:
```bash
docker build -t globe .
docker run -it --rm globe -s
```

## Run

To get a full listing of available features and options, show the `--help`
information with:
```
globe -h
```

Display a globe in *screensaver mode* using the `-s` option. 
```
globe -s 
```

It's kind of boring. Let's add some camera rotation to make it look more
alive:
```
globe -sc2
```

Now let's also enable the night side and rotate the globe on its axis:
```
globe -snc2 -g10
```

If you want to adjust things at runtime check out the *interactive mode*.
Here you can pan the globe around using either the mouse or keyboard arrows:
```
globe -i
```

Use `+` and `-` to control the globe rotation speed, `,` and `.` to control
the camera rotation speed, `PgUp` and `PgDown` to control the camera zoom,
`n` to toggle displaying globe's night side.

Settings we used on the *screensaver mode* also work:
```
globe -inc2 -g10
```

Last but not least there is the *listing mode*. It allows you to pass location
coordinates to the program and see them shown one by one on the globe.
Currently, it only supports a very basic input format. Here's an example:
```
echo "0,0.5;0.1,0.5;0.3,0.5;0.5,0.5;0.7,0.5" | globe -p
```

If you're feeling creative, you can also load custom textures, like so:
```
globe -in --texture ./path-to-texture --texture-night ./path-to-night-texture
```

## Use the library

To use `globe` within your Rust project, add it to your dependencies:
```
[dependencies]
globe = "0.2.0"
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
let (size_x, size_y) = canvas.get_size();
// default character size is 4 by 8
for i in 0..size_y / 8 {
    for j in 0..size_x / 4 {
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
