use std::num::ParseFloatError;
use std::path::PathBuf;
use std::str::FromStr;

use colorgrad::{Color, ParseColorError};

#[derive(Clone)]
pub enum BlendMode {
    Rgb,
    LinearRgb,
    Oklab,
    Lab,
}

impl FromStr for BlendMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rgb" => Ok(Self::Rgb),
            "linear-rgb" => Ok(Self::LinearRgb),
            "oklab" => Ok(Self::Oklab),
            "lab" => Ok(Self::Lab),
            _ => Err(format!(
                "Invalid --blend-mode '{s}' [pick from: rgb, linear-rgb, oklab, lab]"
            )),
        }
    }
}

#[derive(Copy, Clone)]
pub enum Interpolation {
    Linear,
    Basis,
    CatmullRom,
}

impl FromStr for Interpolation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "linear" => Ok(Self::Linear),
            "basis" => Ok(Self::Basis),
            "catmull-rom" => Ok(Self::CatmullRom),
            _ => Err(format!(
                "Invalid --interpolation '{s}' [pick from: linear, basis, catmull-rom]"
            )),
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum OutputColor {
    Hex,
    Rgb,
    Rgb255,
    Hsl,
    Hsv,
    Hwb,
}

impl FromStr for OutputColor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "hex" => Ok(Self::Hex),
            "rgb" => Ok(Self::Rgb),
            "rgb255" => Ok(Self::Rgb255),
            "hsl" => Ok(Self::Hsl),
            "hsv" => Ok(Self::Hsv),
            "hwb" => Ok(Self::Hwb),
            _ => Err(format!(
                "Invalid --format '{s}' [pick from: hex, rgb, rgb255, hsl, hsv, hwb]"
            )),
        }
    }
}

pub const PRESET_NAMES: [&str; 38] = [
    "blues",
    "br-bg",
    "bu-gn",
    "bu-pu",
    "cividis",
    "cool",
    "cubehelix",
    "gn-bu",
    "greens",
    "greys",
    "inferno",
    "magma",
    "or-rd",
    "oranges",
    "pi-yg",
    "plasma",
    "pr-gn",
    "pu-bu",
    "pu-bu-gn",
    "pu-or",
    "pu-rd",
    "purples",
    "rainbow",
    "rd-bu",
    "rd-gy",
    "rd-pu",
    "rd-yl-bu",
    "rd-yl-gn",
    "reds",
    "sinebow",
    "spectral",
    "turbo",
    "viridis",
    "warm",
    "yl-gn",
    "yl-gn-bu",
    "yl-or-br",
    "yl-or-rd",
];

const VERSION: &str = "\
gradient v0.5.0
";

const HELP: &str = "\
A command line tool for playing with color gradients

Usage: gradient [OPTIONS]

Options:
  -W, --width <NUM>               Gradient display width [default: terminal width]
  -H, --height <NUM>              Gradient display height [default: 2]
  -b, --background <COLOR>        Background color [default: checkerboard]
      --cb-color <COLOR> <COLOR>  Checkerboard color
  -t, --take <NUM>                Get N colors evenly spaced across gradient
  -s, --sample <FLOAT>...         Get color(s) at specific position
  -o, --format <FORMAT>           Output color format [possible values: hex, rgb, rgb255, hsl, hsv, hwb]
  -a, --array                     Print colors from --take or --sample, as array
      --named-colors              Lists all CSS named colors
  -h, --help                      Print help (see more with '--help')
      --version                   Print version

PRESET GRADIENT:
  -l, --list-presets   Lists all available preset gradient names
  -p, --preset <NAME>  Use the preset gradient

CUSTOM GRADIENT:
  -c, --custom <COLOR>...         Create custom gradient with the specified colors
  -P, --position <FLOAT>...       Custom gradient color position
  -C, --css <CSS-GRADIENT>        Custom gradient using CSS gradient format
  -m, --blend-mode <COLOR-SPACE>  Custom gradient blending mode [default: oklab] [possible values: rgb,
                                  linear-rgb, oklab, lab]
  -i, --interpolation <MODE>      Custom gradient interpolation mode [default: catmull-rom] [possible values:
                                  linear, basis, catmull-rom]

GRADIENT FILE:
      --ggr-bg <COLOR>  GGR background color [default: white]
      --ggr-fg <COLOR>  GGR foreground color [default: black]
      --svg-id <ID>     Pick SVG gradient by ID
  -f, --file <FILE>...  Read gradient from SVG or GIMP gradient (ggr) file(s)

\x1B[1mCOLOR\x1B[0m can be specified using CSS color format <https://www.w3.org/TR/css-color-4/>.
";

const EXTRA_HELP: &str = "
\x1B[1;4mUsage Examples:\x1B[0m
  Display preset gradient

      \x1B[1m$\x1B[0m gradient --preset rainbow

  Get 15 colors from preset gradient

      \x1B[1m$\x1B[0m gradient --preset spectral --take 15

  Create & display custom gradient

      \x1B[1m$\x1B[0m gradient --custom deeppink gold seagreen

  Create custom gradient & get 20 colors

      \x1B[1m$\x1B[0m gradient --custom ff00ff 'rgb(50,200,70)' 'hwb(195,0,0.5)' --take 20

\x1B[1;4mRepository:\x1B[0m
  URL: https://github.com/mazznoer/gradient-rs
";

#[derive(Default)]
pub struct Opt {
    pub list_presets: bool,
    pub preset: Option<String>,
    pub custom: Option<Vec<Color>>,
    pub position: Option<Vec<f32>>,
    pub css: Option<String>,
    pub blend_mode: Option<BlendMode>,
    pub interpolation: Option<Interpolation>,
    pub ggr_bg: Option<Color>,
    pub ggr_fg: Option<Color>,
    pub svg_id: Option<String>,
    pub file: Option<Vec<PathBuf>>,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub background: Option<Color>,
    pub cb_color: Option<[Color; 2]>,
    pub take: Option<usize>,
    pub sample: Option<Vec<f32>>,
    pub format: Option<OutputColor>,
    pub array: bool,
    pub named_colors: bool,
}

#[rustfmt::skip]
pub fn parse_args() -> Result<Opt, lexopt::Error> {
    use std::process::exit;
    use lexopt::prelude::*;

    let mut opt: Opt = Default::default();

    let mut parser = lexopt::Parser::from_env();

    while let Some(arg) = parser.next()? {
        match arg {
            Short('h') => {
                print!("{HELP}");
                exit(0);
            }
            Long("help") => {
                print!("{HELP}");
                print!("{EXTRA_HELP}");
                exit(0);
            }
            Long("version") => {
                print!("{VERSION}");
                exit(0);
            }
            Short('l') | Long("list-presets") => {
                opt.list_presets = true;
            }
            Short('p') | Long("preset") => {
                if opt.custom.is_some() || opt.css.is_some() {
                    return Err("choose one: --preset, --custom, --css".into());
                }
                opt.preset = Some(parser.value()?.parse_with(parse_preset_name)?);
            }
            Short('c') | Long("custom") => {
                if opt.preset.is_some() || opt.css.is_some() {
                    return Err("choose one: --preset, --custom, --css".into());
                }
                for s in parser.values()? {
                    let v = s.parse_with(parse_colors)?;
                    if let Some(ref mut colors) = opt.custom {
                        colors.extend(v);
                    } else {
                        opt.custom = Some(v);
                    }
                }
            }
            Short('P') | Long("position") => {
                for s in parser.values()? {
                    let v = s.parse_with(parse_floats)?;
                    if let Some(ref mut position) = opt.position {
                        position.extend(v);
                    } else {
                        opt.position = Some(v);
                    }
                }
            }
            Short('C') | Long("css") => {
                if opt.custom.is_some() || opt.preset.is_some() {
                    return Err("choose one: --preset, --custom, --css".into());
                }
                opt.css = Some(parser.value()?.parse()?);
            }
            Short('m') | Long("blend-mode") => {
                opt.blend_mode = Some(parser.value()?.parse()?);
            }
            Short('i') | Long("interpolation") => {
                opt.interpolation = Some(parser.value()?.parse()?);
            }
            Long("ggr-bg") => {
                opt.ggr_bg = Some(parser.value()?.parse()?);
            }
            Long("ggr-fg") => {
                opt.ggr_fg = Some(parser.value()?.parse()?);
            }
            Long("svg-id") => {
                opt.svg_id = Some(parser.value()?.parse()?);
            }
            Short('f') | Long("file") => {
                let res: Result<Vec<_>,_> = parser.values()?.map(|s|s.parse()).collect();
                opt.file = Some(res?);
            }
            Short('W') | Long("width") => {
                opt.width = Some(parser.value()?.parse()?);
            }
            Short('H') | Long("height") => {
                opt.height = Some(parser.value()?.parse()?);
            }
            Short('b') | Long("background") => {
                opt.background = Some(parser.value()?.parse()?);
            }
            Long("cb-color") => {
                opt.cb_color = Some([
                    parser.value()?.parse()?,
                    parser.value()?.parse()?,
                ]);
            }
            Short('t') | Long("take") => {
                if opt.sample.is_some() {
                    return Err("--take cannot be used with --sample".into());
                }
                opt.take = Some(parser.value()?.parse()?);
            }
            Short('s') | Long("sample") => {
                if opt.take.is_some() {
                    return Err("--take cannot be used with --sample".into());
                }
                for s in parser.values()? {
                    let v = s.parse_with(parse_floats)?;
                    if let Some(ref mut sample) = opt.sample {
                        sample.extend(v);
                    } else {
                        opt.sample = Some(v);
                    }
                }
            }
            Short('o') | Long("format") => {
                opt.format = Some(parser.value()?.parse()?);
            }
            Short('a') | Long("array") => {
                opt.array = true;
            }
            Long("named-colors") => {
                opt.named_colors = true;
            }
            _ => return Err(arg.unexpected()),
        }
    }

    Ok(opt)
}

fn parse_preset_name(s: &str) -> Result<String, String> {
    let s = s.to_lowercase();
    if PRESET_NAMES.contains(&s.as_ref()) {
        return Ok(s);
    }
    Err("invalid preset name, try --list-presets".into())
}

fn parse_floats(s: &str) -> Result<Vec<f32>, ParseFloatError> {
    s.split(',').map(|s| s.trim().parse()).collect()
}

fn parse_colors(s: &str) -> Result<Vec<Color>, ParseColorError> {
    let mut colors = Vec::new();
    let mut start = 0;
    let mut inside = false;

    for (i, c) in s.chars().enumerate() {
        if c == ',' && !inside {
            colors.push(s[start..i].parse()?);
            start = i + 1;
        } else if c == '(' {
            inside = true;
        } else if c == ')' {
            inside = false;
        }
    }
    colors.push(s[start..].parse()?);
    Ok(colors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_name_test() {
        assert_eq!(parse_preset_name("rainbow"), Ok("rainbow".into()));
        assert_eq!(parse_preset_name("Rainbow"), Ok("rainbow".into()));
        assert_eq!(parse_preset_name("PLASMA"), Ok("plasma".into()));

        assert!(parse_preset_name("sky").is_err());
    }

    #[test]
    fn parse_floats_test() {
        assert_eq!(parse_floats("90"), Ok(vec![90.0]));
        assert_eq!(parse_floats("0,0.5,1"), Ok(vec![0.0, 0.5, 1.0]));
        assert_eq!(
            parse_floats(" 12.7 , 0.56 ,-0.9"),
            Ok(vec![12.7, 0.56, -0.9])
        );
        assert_eq!(parse_floats("-2.3, 0.73 "), Ok(vec![-2.3, 0.73]));

        assert!(parse_floats("1,0.5,6p,8").is_err());
        assert!(parse_floats("").is_err());
    }

    #[test]
    fn parse_colors_test() {
        let res = parse_colors("rgb(0,255, 0), #ff0, rgba(100%, 0%, 0%, 100%), blue").unwrap();

        assert_eq!(res.len(), 4);
        assert_eq!(res[0].to_hex_string(), "#00ff00");
        assert_eq!(res[1].to_hex_string(), "#ffff00");
        assert_eq!(res[2].to_hex_string(), "#ff0000");
        assert_eq!(res[3].to_hex_string(), "#0000ff");

        assert!(parse_colors("red, rgb(90,20,)").is_err());
    }
}
