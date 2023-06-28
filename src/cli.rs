use clap::{Parser, ValueEnum};
use colorgrad::Color;
use std::path::PathBuf;

#[derive(Clone, ValueEnum)]
pub enum BlendMode {
    Rgb,
    LinearRgb,
    Oklab,
    Lab,
}

#[derive(Clone, ValueEnum)]
pub enum Interpolation {
    Linear,
    Basis,
    CatmullRom,
}

#[derive(Copy, Clone, ValueEnum)]
pub enum OutputColor {
    Hex,
    Rgb,
    Rgb255,
    Hsl,
    Hsv,
    Hwb,
}

const EXTRA_HELP: &str =
    "\x1B[1mCOLOR\x1B[0m can be specified using CSS color format <https://www.w3.org/TR/css-color-4/>.";

const EXTRA_LONG_HELP: &str = "\x1B[1;4mUsage Examples:\x1B[0m
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

#[derive(Clone, Default, Parser)]
#[command(name = "gradient", author, version, about, after_help = EXTRA_HELP, after_long_help = EXTRA_LONG_HELP)]
#[command(arg_required_else_help(true))]
pub struct Opt {
    /// Lists all available preset gradient names
    #[arg(short = 'l', long, help_heading = Some("PRESET GRADIENT"))]
    pub list_presets: bool,

    /// Use the preset gradient
    #[arg(short = 'p', long, value_name = "NAME", help_heading = Some("PRESET GRADIENT"))]
    pub preset: Option<String>,

    /// Create custom gradient with the specified colors
    #[arg(short = 'c', long, num_args = 1.., value_name = "COLOR", conflicts_with = "preset", help_heading = Some("CUSTOM GRADIENT"))]
    pub custom: Option<Vec<Color>>,

    /// Custom gradient color position
    #[arg(short = 'P', long, num_args = 2.., value_name = "FLOAT", help_heading = Some("CUSTOM GRADIENT"))]
    pub position: Option<Vec<f32>>,

    /// Custom gradient blending mode [default: oklab]
    #[arg(short = 'm', long, value_enum, value_name = "COLOR-SPACE", help_heading = Some("CUSTOM GRADIENT"))]
    pub blend_mode: Option<BlendMode>,

    /// Custom gradient interpolation mode [default: catmull-rom]
    #[arg(short = 'i', long, value_enum, value_name = "MODE", help_heading = Some("CUSTOM GRADIENT"))]
    pub interpolation: Option<Interpolation>,

    /// GGR background color [default: white]
    #[arg(long, value_name = "COLOR", help_heading = Some("GRADIENT FILE"))]
    pub ggr_bg: Option<Color>,

    /// GGR foreground color [default: black]
    #[arg(long, value_name = "COLOR", help_heading = Some("GRADIENT FILE"))]
    pub ggr_fg: Option<Color>,

    /// Pick SVG gradient by ID
    #[arg(long, value_name = "ID", help_heading = Some("GRADIENT FILE"))]
    pub svg_id: Option<String>,

    /// Read gradient from SVG or GIMP gradient (ggr) file(s)
    #[arg(
        short = 'f',
        long,
        num_args = 1..,
        value_name = "FILE",
        value_parser = clap::value_parser!(PathBuf),
        help_heading = Some("GRADIENT FILE")
    )]
    pub file: Option<Vec<PathBuf>>,

    /// Gradient display width [default: terminal width]
    #[arg(short = 'W', long, value_name = "NUM")]
    pub width: Option<usize>,

    /// Gradient display height [default: 2]
    #[arg(short = 'H', long, value_name = "NUM")]
    pub height: Option<usize>,

    /// Background color [default: checkerboard]
    #[arg(short = 'b', long, value_name = "COLOR")]
    pub background: Option<Color>,

    /// Checkerboard color
    #[arg(long, number_of_values = 2, value_name = "COLOR")]
    pub cb_color: Option<Vec<Color>>,

    /// Get N colors evenly spaced across gradient
    #[arg(short = 't', long, value_name = "NUM", conflicts_with = "sample")]
    pub take: Option<usize>,

    /// Get color(s) at specific position
    #[arg(short = 's', long, value_name = "FLOAT", num_args = 1..)]
    pub sample: Option<Vec<f32>>,

    /// Output color format
    #[arg(short = 'o', long, value_enum, value_name = "FORMAT")]
    pub format: Option<OutputColor>,

    /// Print colors from --take or --sample, as array
    #[arg(short = 'a', long)]
    pub array: bool,
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Opt::command().debug_assert()
}
