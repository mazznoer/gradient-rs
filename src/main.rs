#![allow(clippy::many_single_char_names)]

use clap::{AppSettings, ArgEnum, Clap};
use colorgrad::{Color, Gradient};
use std::{ffi::OsStr, fs::File, io::BufReader, path::PathBuf, process::exit};

mod svg_gradient;
use svg_gradient::parse_svg;

const LEFT_HALF_BLOCK: char = '\u{258C}';

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
    Rgb255,
    Hsl,
    Hsv,
    Hwb,
}

const PRESET_NAMES: [&str; 38] = [
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

const EXTRA_HELP: &str =
    "COLOR can be specified using CSS color format <https://www.w3.org/TR/css-color-4/>.";

const EXTRA_LONG_HELP: &str = "EXAMPLES:
Display preset gradient

    gradient -p rainbow

Get 15 colors from preset gradient

    gradient -p spectral -t 15

Create & display custom gradient

    gradient -c deeppink gold seagreen

Create custom gradient & get 20 colors

    gradient -c ff00ff 'rgb(50,200,70)' 'hwb(195,0,0.5)' -t 20

REPOSITORY: <https://github.com/mazznoer/gradient-rs>
";

#[derive(Debug, Clap)]
#[clap(name = "gradient", author, version, about, after_help = EXTRA_HELP, after_long_help = EXTRA_LONG_HELP, setting = AppSettings::ArgRequiredElseHelp)]
struct Opt {
    /// Lists all available preset gradient names
    #[clap(short = 'l', long, help_heading = Some("PRESET GRADIENT"))]
    list_presets: bool,

    /// Use the preset gradient
    #[clap(short = 'p', long, value_name = "NAME", help_heading = Some("PRESET GRADIENT"))]
    preset: Option<String>,

    /// Create custom gradient with the specified colors
    #[clap(short = 'c', long, parse(try_from_str = parse_color), multiple_values = true, multiple_occurrences = true, min_values = 1, value_name = "COLOR", conflicts_with = "preset", help_heading = Some("CUSTOM GRADIENT"))]
    custom: Option<Vec<Color>>,

    /// Custom gradient color position
    #[clap(short = 'P', long, multiple_values = true, multiple_occurrences = true, min_values = 2, value_name = "FLOAT", help_heading = Some("CUSTOM GRADIENT"))]
    position: Option<Vec<f64>>,

    /// Custom gradient blending mode [default: oklab]
    #[clap(short = 'm', long, arg_enum, value_name = "COLOR-SPACE", help_heading = Some("CUSTOM GRADIENT"))]
    blend_mode: Option<BlendMode>,

    /// Custom gradient interpolation mode [default: catmull-rom]
    #[clap(short = 'i', long, arg_enum, value_name = "MODE", help_heading = Some("CUSTOM GRADIENT"))]
    interpolation: Option<Interpolation>,

    /// GGR background color [default: white]
    #[clap(long, parse(try_from_str = parse_color), value_name = "COLOR", help_heading = Some("GRADIENT FILE"))]
    ggr_bg: Option<Color>,

    /// GGR foreground color [default: black]
    #[clap(long, parse(try_from_str = parse_color), value_name = "COLOR", help_heading = Some("GRADIENT FILE"))]
    ggr_fg: Option<Color>,

    /// Pick SVG gradient by ID
    #[clap(long, value_name = "ID", help_heading = Some("GRADIENT FILE"))]
    svg_id: Option<String>,

    /// Read gradient from SVG or GIMP gradient (ggr) file(s)
    #[clap(
        short = 'f',
        long,
        min_values = 1,
        value_name = "FILE",
        parse(from_os_str),
        help_heading = Some("GRADIENT FILE")
    )]
    file: Option<Vec<PathBuf>>,

    /// Gradient display width [default: terminal width]
    #[clap(short = 'W', long, value_name = "NUM")]
    width: Option<usize>,

    /// Gradient display height [default: 2]
    #[clap(short = 'H', long, value_name = "NUM")]
    height: Option<usize>,

    /// Background color [default: checkerboard]
    #[clap(short = 'b', long, parse(try_from_str = parse_color), value_name = "COLOR")]
    background: Option<Color>,

    /// Checkerboard color
    #[clap(long, number_of_values = 2, parse(try_from_str = parse_color), value_name = "COLOR")]
    cb_color: Option<Vec<Color>>,

    /// Get N colors evenly spaced across gradient
    #[clap(short = 't', long, value_name = "NUM", conflicts_with = "sample")]
    take: Option<usize>,

    /// Get color(s) at specific position
    #[clap(
        short = 's',
        long,
        value_name = "FLOAT",
        multiple_values = true,
        multiple_occurrences = true,
        min_values = 1
    )]
    sample: Option<Vec<f64>>,

    /// Output color format
    #[clap(short = 'o', long, arg_enum, value_name = "FORMAT")]
    format: Option<OutputColor>,
}

#[derive(Debug)]
struct Config {
    is_stdout: bool,
    use_solid_bg: bool,
    background: Color,
    cb_color: Vec<Color>,
    term_width: usize,
    width: usize,
    height: usize,
    output_format: OutputColor,
}

#[derive(Debug)]
enum OutputMode {
    Gradient,
    ColorsN(usize),
    ColorsSample(Vec<f64>),
}

fn main() {
    let opt = Opt::parse();

    if opt.list_presets {
        for name in &PRESET_NAMES {
            println!("{}", name);
        }
        exit(0);
    }

    let term_width = if let Some((terminal_size::Width(w), _)) = terminal_size::terminal_size() {
        w as usize
    } else {
        80
    };

    let ggr_bg_color = opt.ggr_bg.unwrap_or_else(|| Color::from_rgb(1.0, 1.0, 1.0));
    let ggr_fg_color = opt.ggr_fg.unwrap_or_else(|| Color::from_rgb(0.0, 0.0, 0.0));

    let cfg = Config {
        is_stdout: atty::is(atty::Stream::Stdout),
        use_solid_bg: opt.background.is_some(),
        background: opt
            .background
            .unwrap_or_else(|| Color::from_rgb(0.0, 0.0, 0.0)),
        cb_color: opt.cb_color.unwrap_or_else(|| {
            vec![
                Color::from_rgb(0.05, 0.05, 0.05),
                Color::from_rgb(0.20, 0.20, 0.20),
            ]
        }),
        term_width,
        width: opt.width.unwrap_or(term_width).max(10).min(term_width),
        height: opt.height.unwrap_or(2).max(1).min(50),
        output_format: opt.format.unwrap_or(OutputColor::Hex),
    };

    let output_mode = if let Some(n) = opt.take {
        OutputMode::ColorsN(n)
    } else if let Some(pos) = opt.sample {
        OutputMode::ColorsSample(pos)
    } else {
        OutputMode::Gradient
    };

    if let Some(name) = opt.preset {
        let grad = match name.to_lowercase().replace("-", "_").as_ref() {
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
            _ => {
                eprintln!("Error: Invalid preset gradient name. Use -l flag to list all preset gradient names.");
                exit(1);
            }
        };

        handle_output(&grad, &output_mode, &cfg);
    }

    if let Some(colors) = opt.custom {
        let pos = opt.position.unwrap_or_else(|| vec![0.0, 1.0]);

        let blend_mode = match opt.blend_mode {
            Some(BlendMode::Rgb) => colorgrad::BlendMode::Rgb,
            Some(BlendMode::LinearRgb) => colorgrad::BlendMode::LinearRgb,
            Some(BlendMode::Hsv) => colorgrad::BlendMode::Hsv,
            _ => colorgrad::BlendMode::Oklab,
        };

        let interpolation = match opt.interpolation {
            Some(Interpolation::Linear) => colorgrad::Interpolation::Linear,
            Some(Interpolation::Basis) => colorgrad::Interpolation::Basis,
            _ => colorgrad::Interpolation::CatmullRom,
        };

        match colorgrad::CustomGradient::new()
            .colors(&colors)
            .domain(&pos)
            .mode(blend_mode)
            .interpolation(interpolation)
            .build()
        {
            Ok(grad) => {
                handle_output(&grad, &output_mode, &cfg);
            }
            Err(err) => {
                eprintln!("Custom gradient error: {}", err);
                exit(1);
            }
        }
    }

    if let Some(files) = opt.file {
        for path in files {
            if let Some(ext) = path.extension().and_then(OsStr::to_str) {
                match ext.to_lowercase().as_ref() {
                    "ggr" => {
                        if cfg.is_stdout {
                            print!("{}", &path.display());
                        }

                        let f = File::open(&path).unwrap();

                        match colorgrad::parse_ggr(BufReader::new(f), &ggr_fg_color, &ggr_bg_color)
                        {
                            Ok((grad, name)) => {
                                if cfg.is_stdout {
                                    println!(" \x1B[1m{}\x1B[0m", name);
                                }

                                handle_output(&grad, &output_mode, &cfg);
                            }
                            Err(err) => {
                                if cfg.is_stdout {
                                    println!("\n  \x1B[31m{}\x1B[39m", err);
                                }
                            }
                        }
                    }
                    "svg" => {
                        let filename = &path.display().to_string();
                        let gradients =
                            parse_svg(path.into_os_string().into_string().unwrap().as_ref());

                        if gradients.is_empty() && cfg.is_stdout {
                            println!("{}", filename);
                            println!("  \x1B[31mNo gradients.\x1B[39m");
                        }

                        for (grad, id) in gradients {
                            let (id, stop) = if let Some(id) = id {
                                if let Some(ref id2) = opt.svg_id {
                                    if &id == id2 {
                                        (format!("#{}", id), true)
                                    } else {
                                        continue;
                                    }
                                } else {
                                    (format!("#{}", id), false)
                                }
                            } else {
                                ("".to_string(), false)
                            };

                            if cfg.is_stdout {
                                println!("{} \x1B[1m{}\x1B[0m", filename, id);
                            }

                            handle_output(&grad, &output_mode, &cfg);

                            if stop {
                                break;
                            }
                        }
                    }
                    _ => continue,
                }
            }
        }
    }

    exit(0);
}

fn parse_color(s: &str) -> Result<Color, colorgrad::ParseColorError> {
    s.parse::<Color>()
}

fn handle_output(grad: &Gradient, mode: &OutputMode, cfg: &Config) {
    match mode {
        OutputMode::Gradient => display_gradient(grad, cfg),
        OutputMode::ColorsN(n) => display_colors_n(grad, *n, cfg),
        OutputMode::ColorsSample(ref pos) => display_colors_sample(grad, pos, cfg),
    }
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

fn format_alpha(a: f64) -> String {
    let s = format!(",{:.2}%", a * 100.0);
    if s.starts_with(",100") {
        return "".to_string();
    }
    s
}

fn format_color(col: &Color, format: OutputColor) -> String {
    match format {
        OutputColor::Hex => col.to_hex_string(),
        OutputColor::Rgb => {
            let (r, g, b, a) = col.rgba();
            format!(
                "rgb({:.2}%,{:.2}%,{:.2}%{})",
                r * 100.0,
                g * 100.0,
                b * 100.0,
                format_alpha(a)
            )
        }
        OutputColor::Rgb255 => {
            let (r, g, b, _) = col.rgba_u8();
            let x = col.rgba();
            format!("rgb({},{},{}{})", r, g, b, format_alpha(x.3))
        }
        OutputColor::Hsl => {
            let (h, s, l, a) = col.to_hsla();
            format!(
                "hsl({:.2},{:.2}%,{:.2}%{})",
                h,
                s * 100.0,
                l * 100.0,
                format_alpha(a)
            )
        }
        OutputColor::Hsv => {
            let (h, s, v, a) = col.to_hsva();
            format!(
                "hsv({:.2},{:.2}%,{:.2}%{})",
                h,
                s * 100.0,
                v * 100.0,
                format_alpha(a)
            )
        }
        OutputColor::Hwb => {
            let (h, w, b, a) = col.to_hwba();
            format!(
                "hwb({:.2},{:.2}%,{:.2}%{})",
                h,
                w * 100.0,
                b * 100.0,
                format_alpha(a)
            )
        }
    }
}

fn display_colors(colors: &[Color], cfg: &Config) {
    if cfg.is_stdout {
        let mut width = cfg.term_width;

        for col in colors {
            let (col, bg) = if cfg.use_solid_bg {
                let c = blend(col, &cfg.background);
                (c.clone(), c)
            } else {
                (col.clone(), blend(col, &Color::from_rgb(0.0, 0.0, 0.0)))
            };

            let s = format_color(&col, cfg.output_format);

            if width < s.len() {
                println!();
                width = cfg.term_width;
            }

            let (r, g, b, _) = bg.rgba_u8();

            let fg = if color_luminance(&bg) < 0.3 {
                (255, 255, 255)
            } else {
                (0, 0, 0)
            };

            print!(
                "\x1B[38;2;{};{};{};48;2;{};{};{}m{}\x1B[39;49m",
                fg.0, fg.1, fg.2, r, g, b, &s
            );
            width -= s.len();

            if width >= 1 {
                print!(" ");
                width -= 1;
            }
        }

        println!();
    } else {
        for col in colors {
            if cfg.use_solid_bg {
                println!(
                    "{}",
                    format_color(&blend(col, &cfg.background), cfg.output_format)
                );
            } else {
                println!("{}", format_color(col, cfg.output_format));
            }
        }
    }
}

fn display_colors_n(grad: &Gradient, n: usize, cfg: &Config) {
    display_colors(&grad.colors(n), cfg);
}

fn display_colors_sample(grad: &Gradient, positions: &[f64], cfg: &Config) {
    let colors = positions.iter().map(|t| grad.at(*t)).collect::<Vec<_>>();
    display_colors(&colors, cfg);
}

fn display_gradient(grad: &Gradient, cfg: &Config) {
    if !cfg.is_stdout {
        return;
    }

    let (dmin, dmax) = grad.domain();
    let w2 = (cfg.width * 2 - 1) as f64;
    let bg_0 = &cfg.cb_color[0];
    let bg_1 = &cfg.cb_color[1];

    for y in 0..cfg.height {
        let mut i = 0;

        for x in 0..cfg.width {
            let bg = if cfg.use_solid_bg {
                &cfg.background
            } else if ((x / 2) & 1) ^ (y & 1) == 0 {
                bg_0
            } else {
                bg_1
            };

            let col1 = grad.at(remap(i as f64, 0.0, w2, dmin, dmax));
            i += 1;

            let col2 = grad.at(remap(i as f64, 0.0, w2, dmin, dmax));
            i += 1;

            let col1 = blend(&col1, bg).rgba_u8();
            let col2 = blend(&col2, bg).rgba_u8();

            print!(
                "\x1B[38;2;{};{};{};48;2;{};{};{}m{}",
                col1.0, col1.1, col1.2, col2.0, col2.1, col2.2, LEFT_HALF_BLOCK
            );
        }

        println!("\x1B[39;49m");
    }
}

// Map t from range [a, b] to range [c, d]
fn remap(t: f64, a: f64, b: f64, c: f64, d: f64) -> f64 {
    (t - a) * ((d - c) / (b - a)) + c
}
