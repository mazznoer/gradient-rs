# gradient

A command-line tool to play with color gradient.

![gradient-cli-tool](docs/images/gradient-cli-1.png)

## Installing

```
$ cargo install gradient
```

## Usage

```
gradient 
A command-line color gradient tool.

USAGE:
    gradient [FLAGS] [OPTIONS]

FLAGS:
    -h, --help            Prints help information
        --list-presets    List preset names
    -V, --version         Prints version information

OPTIONS:
    -b, --background <COLOR>          Background color [default: checkerboard]
    -m, --blend-mode <COLOR-SPACE>    Custom gradient blending mode [default: rgb] [possible values:
                                      rgb, linear-rgb, hsv, oklab]
    -c, --custom <COLOR>...           Custom gradient
    -H, --height <NUM>                Gradient display height [default: 2]
    -i, --interpolation <MODE>        Custom gradient interpolation mode [default: linear] [possible
                                      values: linear, basis, catmull-rom]
    -p, --preset <NAME>               Preset gradients
    -s, --sample <FLOAT>...           Get color at t
    -t, --take <NUM>                  Get n colors evenly spaced across gradient
    -W, --width <NUM>                 Gradient display width [default: terminal width]
```

