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

To use `globe` within your Rust project, first add it to your dependencies:
```
[dependencies]
globe = "0.1.0"
```


## Credits

Rendering math based on 
[C++ code by DinoZ1729](https://github.com/DinoZ1729/Earth).