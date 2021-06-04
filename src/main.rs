#![allow(clippy::many_single_char_names)]

use clap::{AppSettings, ArgEnum, Clap};
use colored::Colorize;
use colorgrad::{Color, Gradient};
use csscolorparser::parse as parse_color;
use std::process::exit;

#[derive(Debug, ArgEnum)]
enum BlendMode {
    Rgb,
    LinearRgb,
    Hsv,
    Oklab,
}

#[derive(Debug, ArgEnum)]
enum Interpolation {
    Linear,
    Basis,
    CatmullRom,
}

#[derive(Debug, Copy, Clone, ArgEnum)]
enum OutputColor {
    Hex,
    Rgb,
    Hsl,
    Hsv,
    Hwb,
}

const PRESET_NAMES: [&str; 38] = [
    "blues",
    "br_bg",
    "bu_gn",
    "bu_pu",
    "cividis",
    "cool",
    "cubehelix",
    "gn_bu",
    "greens",
    "greys",
    "inferno",
    "magma",
    "or_rd",
    "oranges",
    "pi_yg",
    "plasma",
    "pr_gn",
    "pu_bu",
    "pu_bu_gn",
    "pu_or",
    "pu_rd",
    "purples",
    "rainbow",
    "rd_bu",
    "rd_gy",
    "rd_pu",
    "rd_yl_bu",
    "rd_yl_gn",
    "reds",
    "sinebow",
    "spectral",
    "turbo",
    "viridis",
    "warm",
    "yl_gn",
    "yl_gn_bu",
    "yl_or_br",
    "yl_or_rd",
];

const EXTRA_HELP: &str = "EXAMPLES:
Display preset gradient

    gradient -p rainbow

Get 15 colors from preset gradient

    gradient -p spectral -t 15

Create & display custom gradient

    gradient -c deeppink gold seagreen

Create custom gradient & get 20 colors

    gradient -c ff00ff 'rgb(50,200,70)' 'hwb(195,0,0.5)' -t 20

REPOSITORY: https://github.com/mazznoer/gradient-rs
";

#[derive(Debug, Clap)]
#[clap(name = "gradient", version, about, after_help = EXTRA_HELP, setting = AppSettings::ArgRequiredElseHelp)]
struct Opt {
    /// List preset gradient names
    #[clap(short = 'l', long, help_heading = Some("PRESET GRADIENT"))]
    list_presets: bool,

    /// Preset gradients
    #[clap(short = 'p', long, possible_values = &PRESET_NAMES, value_name = "NAME", hide_possible_values = true, help_heading = Some("PRESET GRADIENT"))]
    preset: Option<String>,

    /// Custom gradient
    #[clap(short = 'c', long, parse(try_from_str = parse_color), multiple = true, min_values = 1, value_name = "COLOR", conflicts_with = "preset", help_heading = Some("CUSTOM GRADIENT"))]
    custom: Option<Vec<Color>>,

    /// Color position
    #[clap(short = 'P', long, multiple = true, min_values = 2, value_name = "FLOAT", help_heading = Some("CUSTOM GRADIENT"))]
    position: Option<Vec<f64>>,

    /// Custom gradient blending mode
    #[clap(short = 'm', long, arg_enum, value_name = "COLOR-SPACE", help_heading = Some("CUSTOM GRADIENT"))]
    blend_mode: Option<BlendMode>,

    /// Custom gradient interpolation mode
    #[clap(short = 'i', long, arg_enum, value_name = "MODE", help_heading = Some("CUSTOM GRADIENT"))]
    interpolation: Option<Interpolation>,

    /// Gradient display width [default: terminal width]
    #[clap(short = 'W', long, value_name = "NUM")]
    width: Option<usize>,

    /// Gradient display height [default: 2]
    #[clap(short = 'H', long, value_name = "NUM")]
    height: Option<usize>,

    /// Background color [default: checkerboard]
    #[clap(short = 'b', long, parse(try_from_str = parse_color), value_name = "COLOR")]
    background: Option<Color>,

    /// Get N colors evenly spaced across gradient
    #[clap(short = 't', long, value_name = "NUM")]
    take: Option<usize>,

    /// Get color(s) at specific position
    #[clap(
        short = 's',
        long,
        value_name = "FLOAT",
        multiple = true,
        min_values = 1,
        conflicts_with = "take"
    )]
    sample: Option<Vec<f64>>,

    /// Output color format
    #[clap(short = 'o', long, arg_enum)]
    format: Option<OutputColor>,
}

#[derive(Debug)]
struct Config {
    is_stdout: bool,
    background: Color,
    term_width: usize,
    output_format: OutputColor,
}

fn main() {
    let opt = Opt::parse();

    if opt.list_presets {
        for name in &PRESET_NAMES {
            println!("{}", name);
        }
        exit(0);
    }

    let (term_width, _) = term_size::dimensions().unwrap_or((80, 0));
    let width = opt.width.unwrap_or(term_width);
    let height = opt.height.unwrap_or(2);

    let base_color = if let Some(ref col) = opt.background {
        col.clone()
    } else {
        Color::from_rgb(0.0, 0.0, 0.0)
    };

    let cfg = Config {
        is_stdout: atty::is(atty::Stream::Stdout),
        background: base_color,
        term_width,
        output_format: opt.format.unwrap_or(OutputColor::Hex),
    };

    if let Some(name) = opt.preset {
        let grad = match name.as_ref() {
            "blues" => colorgrad::blues(),
            "br_bg" => colorgrad::br_bg(),
            "bu_gn" => colorgrad::bu_gn(),
            "bu_pu" => colorgrad::bu_pu(),
            "cividis" => colorgrad::cividis(),
            "cool" => colorgrad::cool(),
            "cubehelix" => colorgrad::cubehelix_default(),
            "gn_bu" => colorgrad::gn_bu(),
            "greens" => colorgrad::greens(),
            "greys" => colorgrad::greys(),
            "inferno" => colorgrad::inferno(),
            "magma" => colorgrad::magma(),
            "or_rd" => colorgrad::or_rd(),
            "oranges" => colorgrad::oranges(),
            "pi_yg" => colorgrad::pi_yg(),
            "plasma" => colorgrad::plasma(),
            "pr_gn" => colorgrad::pr_gn(),
            "pu_bu" => colorgrad::pu_bu(),
            "pu_bu_gn" => colorgrad::pu_bu_gn(),
            "pu_or" => colorgrad::pu_or(),
            "pu_rd" => colorgrad::pu_rd(),
            "purples" => colorgrad::purples(),
            "rainbow" => colorgrad::rainbow(),
            "rd_bu" => colorgrad::rd_bu(),
            "rd_gy" => colorgrad::rd_gy(),
            "rd_pu" => colorgrad::rd_pu(),
            "rd_yl_bu" => colorgrad::rd_yl_bu(),
            "rd_yl_gn" => colorgrad::rd_yl_gn(),
            "reds" => colorgrad::reds(),
            "sinebow" => colorgrad::sinebow(),
            "spectral" => colorgrad::spectral(),
            "turbo" => colorgrad::turbo(),
            "viridis" => colorgrad::viridis(),
            "warm" => colorgrad::warm(),
            "yl_gn" => colorgrad::yl_gn(),
            "yl_gn_bu" => colorgrad::yl_gn_bu(),
            "yl_or_br" => colorgrad::yl_or_br(),
            "yl_or_rd" => colorgrad::yl_or_rd(),
            _ => colorgrad::greys(),
        };

        if let Some(n) = opt.take {
            display_colors_n(&grad, n, &cfg);
        } else if let Some(ref pos) = opt.sample {
            display_colors_sample(&grad, &pos, &cfg);
        } else if let Some(ref _bg_color) = opt.background {
            display_gradient(&grad, width, height, &cfg);
        } else {
            display_gradient_checkerboard(&grad, width, height, &cfg);
        }
    }

    if let Some(colors) = opt.custom {
        let pos = opt.position.unwrap_or_else(|| vec![0.0, 1.0]);

        let blend_mode = match opt.blend_mode {
            Some(BlendMode::LinearRgb) => colorgrad::BlendMode::LinearRgb,
            Some(BlendMode::Hsv) => colorgrad::BlendMode::Hsv,
            Some(BlendMode::Oklab) => colorgrad::BlendMode::Oklab,
            _ => colorgrad::BlendMode::Rgb,
        };

        let interpolation = match opt.interpolation {
            Some(Interpolation::CatmullRom) => colorgrad::Interpolation::CatmullRom,
            Some(Interpolation::Basis) => colorgrad::Interpolation::Basis,
            _ => colorgrad::Interpolation::Linear,
        };

        match colorgrad::CustomGradient::new()
            .colors(&colors)
            .domain(&pos)
            .mode(blend_mode)
            .interpolation(interpolation)
            .build()
        {
            Ok(grad) => {
                if let Some(n) = opt.take {
                    display_colors_n(&grad, n, &cfg);
                } else if let Some(ref pos) = opt.sample {
                    display_colors_sample(&grad, &pos, &cfg);
                } else if let Some(ref _bg_color) = opt.background {
                    display_gradient(&grad, width, height, &cfg);
                } else {
                    display_gradient_checkerboard(&grad, width, height, &cfg);
                }
            }
            Err(err) => {
                eprintln!("Custom gradient error: {}", err);
                exit(1);
            }
        }
    }

    exit(0);
}

fn blend(color: &Color, bg: &Color) -> Color {
    let col = color.rgba();
    let bg = bg.rgba();
    Color::from_rgb(
        ((1.0 - col.3) * bg.0) + (col.3 * col.0),
        ((1.0 - col.3) * bg.1) + (col.3 * col.1),
        ((1.0 - col.3) * bg.2) + (col.3 * col.2),
    )
}

fn color_luminance(col: &Color) -> f64 {
    // http://www.w3.org/TR/2008/REC-WCAG20-20081211/#relativeluminancedef

    fn lum(t: f64) -> f64 {
        if t <= 0.03928 {
            t / 12.92
        } else {
            ((t + 0.055) / 1.055).powf(2.4)
        }
    }

    let (r, g, b, _) = col.rgba();
    0.2126 * lum(r) + 0.7152 * lum(g) + 0.0722 * lum(b)
}

fn format_color(col: &Color, format: OutputColor) -> String {
    match format {
        OutputColor::Hex => col.to_hex_string(),
        OutputColor::Rgb => col.to_rgb_string(),
        OutputColor::Hsl => {
            let (h, s, l, a) = col.to_hsla();
            if a < 1.0 {
                format!("hsl({},{}%,{}%,{})", h, s * 100.0, l * 100.0, a)
            } else {
                format!("hsl({},{}%,{}%)", h, s * 100.0, l * 100.0)
            }
        }
        OutputColor::Hsv => {
            let (h, s, v, a) = col.to_hsva();
            if a < 1.0 {
                format!("hsv({},{}%,{}%,{})", h, s * 100.0, v * 100.0, a)
            } else {
                format!("hsv({},{}%,{}%)", h, s * 100.0, v * 100.)
            }
        }
        OutputColor::Hwb => {
            let (h, w, b, a) = col.to_hwba();
            if a < 1.0 {
                format!("hwb({},{}%,{}%,{})", h, w * 100.0, b * 100.0, a)
            } else {
                format!("hwb({},{}%,{}%)", h, w * 100.0, b * 100.0)
            }
        }
    }
}

fn display_colors(colors: &[Color], cfg: &Config) {
    if cfg.is_stdout {
        let mut width = cfg.term_width;

        for col in colors {
            let s = format_color(&col, cfg.output_format);

            if width < s.len() {
                println!();
                width = cfg.term_width;
            }

            let c = blend(&col, &cfg.background);
            let (r, g, b, _) = c.rgba_u8();

            let fg = if color_luminance(&c) < 0.3 {
                (255, 255, 255)
            } else {
                (0, 0, 0)
            };

            print!("{}", &s.truecolor(fg.0, fg.1, fg.2).on_truecolor(r, g, b));
            width -= s.len();

            if width >= 1 {
                print!(" ");
                width -= 1;
            }
        }

        println!();
    } else {
        for col in colors {
            println!("{}", format_color(&col, cfg.output_format));
        }
    }
}

fn display_colors_n(grad: &Gradient, n: usize, cfg: &Config) {
    display_colors(&grad.colors(n), &cfg);
}

fn display_colors_sample(grad: &Gradient, positions: &[f64], cfg: &Config) {
    let colors = positions.iter().map(|t| grad.at(*t)).collect::<Vec<_>>();
    display_colors(&colors, &cfg);
}

fn display_gradient(grad: &Gradient, w: usize, h: usize, cfg: &Config) {
    if !cfg.is_stdout {
        return;
    }

    let (dmin, dmax) = grad.domain();

    for _ in 0..h {
        for i in 0..w {
            let col = grad.at(remap(i as f64, 0.0, w as f64, dmin, dmax));
            let (r, g, b, _) = blend(&col, &cfg.background).rgba_u8();
            print!("{}", " ".on_truecolor(r, g, b));
        }

        println!();
    }
}

fn display_gradient_checkerboard(grad: &Gradient, w: usize, h: usize, cfg: &Config) {
    if !cfg.is_stdout {
        return;
    }

    let (dmin, dmax) = grad.domain();
    let bg_0 = Color::from_rgb(0.05, 0.05, 0.05);
    let bg_1 = Color::from_rgb(0.20, 0.20, 0.20);

    for y in 0..h {
        for x in 0..w {
            let col = grad.at(remap(x as f64, 0.0, w as f64, dmin, dmax));

            let bg = if ((x / 2) & 1) ^ (y & 1) == 1 {
                &bg_0
            } else {
                &bg_1
            };

            let (r, g, b, _) = blend(&col, &bg).rgba_u8();
            print!("{}", " ".on_truecolor(r, g, b));
        }

        println!();
    }
}

// Map t from range [a, b] to range [c, d]
fn remap(t: f64, a: f64, b: f64, c: f64, d: f64) -> f64 {
    (t - a) * ((d - c) / (b - a)) + c
}
