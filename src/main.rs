#![allow(clippy::many_single_char_names)]

use clap::{ArgEnum, Parser};
use colorgrad::{Color, Gradient};
use std::io::{self, BufReader, Write};
use std::{ffi::OsStr, fs::File, path::PathBuf, process::exit};

mod svg_gradient;
use svg_gradient::parse_svg;

#[derive(Clone, ArgEnum)]
enum BlendMode {
    Rgb,
    LinearRgb,
    Hsv,
    Oklab,
}

#[derive(Clone, ArgEnum)]
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

#[derive(Clone, Default, Parser)]
#[clap(name = "gradient", author, version, about, after_help = EXTRA_HELP, after_long_help = EXTRA_LONG_HELP)]
#[clap(arg_required_else_help(true))]
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

    /// Print colors from --take or --sample, as array
    #[clap(short = 'a', long)]
    array: bool,
}

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

#[derive(PartialEq)]
enum OutputMode {
    Gradient,
    ColorsN,
    ColorsSample,
}

const GGR_SAMPLE: &str = "GIMP Gradient
Name: Neon Green
4
0.000000 0.672788 0.699499 0.000000 1.000000 0.000000 0.000000 0.129412 1.000000 0.000000 0.901961 1 0
0.699499 0.737062 0.774624 0.129412 1.000000 0.000000 0.901961 0.823529 1.000000 0.807843 1.000000 1 0
0.774624 0.812187 0.849750 0.823529 1.000000 0.807843 1.000000 0.196078 1.000000 0.000000 0.901961 1 0
0.849750 0.874791 1.000000 0.196078 1.000000 0.000000 0.901961 0.031373 1.000000 0.000000 0.000000 1 0";

struct GradientApp {
    opt: Opt,
    cfg: Config,
    stdout: io::Stdout,
    output_mode: OutputMode,
}

impl GradientApp {
    fn new(opt: Opt, stdout: io::Stdout) -> Self {
        let term_width = if let Some((terminal_size::Width(w), _)) = terminal_size::terminal_size()
        {
            Some(w as usize)
        } else {
            None
        };

        let background = if let Some(ref c) = opt.background {
            c.clone()
        } else {
            Color::from_rgb(0.0, 0.0, 0.0)
        };

        let cb_color = if let Some(ref c) = opt.cb_color {
            c.clone()
        } else {
            vec![
                Color::from_rgb(0.05, 0.05, 0.05),
                Color::from_rgb(0.20, 0.20, 0.20),
            ]
        };

        let width = opt
            .width
            .unwrap_or_else(|| term_width.unwrap_or(80))
            .max(10)
            .min(term_width.unwrap_or(1000));

        let cfg = Config {
            is_stdout: atty::is(atty::Stream::Stdout),
            use_solid_bg: opt.background.is_some(),
            background,
            cb_color,
            term_width: term_width.unwrap_or(80),
            width,
            height: opt.height.unwrap_or(2).max(1).min(50),
            output_format: opt.format.unwrap_or(OutputColor::Hex),
        };

        let output_mode = if opt.take.is_some() {
            OutputMode::ColorsN
        } else if opt.sample.is_some() {
            OutputMode::ColorsSample
        } else {
            OutputMode::Gradient
        };

        Self {
            opt,
            cfg,
            output_mode,
            stdout,
        }
    }

    fn run(&mut self) -> io::Result<i32> {
        if self.opt.list_presets {
            for name in &PRESET_NAMES {
                writeln!(self.stdout, "{}", name)?;
            }

            return Ok(0);
        }

        if self.opt.preset.is_some() {
            self.preset_gradient()?;
            return Ok(0);
        }

        if self.opt.custom.is_some() {
            self.custom_gradient()?;
            return Ok(0);
        }

        if self.opt.file.is_some() {
            self.file_gradient()?;
            return Ok(0);
        }

        writeln!(
            self.stdout,
            "\x1B[38;5;9merror:\x1B[39m Specify gradient using -p / --preset, -c / --custom or -f / --file\n"
        )?;

        example_help()?;
        Ok(1)
    }

    fn preset_gradient(&mut self) -> io::Result<i32> {
        let grad = match self
            .opt
            .preset
            .as_ref()
            .unwrap()
            .to_lowercase()
            .replace("-", "_")
            .as_ref()
        {
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
                writeln!(io::stderr(), "Error: Invalid preset gradient name. Use -l flag to list all preset gradient names.")?;
                return Ok(1);
            }
        };

        self.handle_output(&grad)?;
        Ok(0)
    }

    fn custom_gradient(&mut self) -> io::Result<i32> {
        let mut gb = colorgrad::CustomGradient::new();

        gb.colors(self.opt.custom.as_ref().unwrap());

        if let Some(ref pos) = self.opt.position {
            gb.domain(pos);
        }

        gb.mode(match self.opt.blend_mode {
            Some(BlendMode::Rgb) => colorgrad::BlendMode::Rgb,
            Some(BlendMode::LinearRgb) => colorgrad::BlendMode::LinearRgb,
            Some(BlendMode::Hsv) => colorgrad::BlendMode::Hsv,
            _ => colorgrad::BlendMode::Oklab,
        });

        gb.interpolation(match self.opt.interpolation {
            Some(Interpolation::Linear) => colorgrad::Interpolation::Linear,
            Some(Interpolation::Basis) => colorgrad::Interpolation::Basis,
            _ => colorgrad::Interpolation::CatmullRom,
        });

        match gb.build() {
            Ok(grad) => {
                self.handle_output(&grad)?;
                Ok(0)
            }

            Err(err) => {
                writeln!(io::stderr(), "Custom gradient error: {}", err)?;
                Ok(1)
            }
        }
    }

    fn file_gradient(&mut self) -> io::Result<i32> {
        let ggr_bg_color = if let Some(ref c) = self.opt.ggr_bg {
            c.clone()
        } else {
            Color::from_rgb(1.0, 1.0, 1.0)
        };

        let ggr_fg_color = if let Some(ref c) = self.opt.ggr_fg {
            c.clone()
        } else {
            Color::from_rgb(0.0, 0.0, 0.0)
        };

        for path in self.opt.file.as_ref().unwrap().clone() {
            if let Some(ext) = path.extension().and_then(OsStr::to_str) {
                match ext.to_lowercase().as_ref() {
                    "ggr" => {
                        if self.cfg.is_stdout || (self.output_mode == OutputMode::Gradient) {
                            write!(self.stdout, "{}", &path.display())?;
                        }

                        let f = File::open(&path).unwrap();

                        match colorgrad::parse_ggr(BufReader::new(f), &ggr_fg_color, &ggr_bg_color)
                        {
                            Ok((grad, name)) => {
                                if self.cfg.is_stdout || (self.output_mode == OutputMode::Gradient)
                                {
                                    writeln!(self.stdout, " \x1B[1m{}\x1B[0m", name)?;
                                }

                                self.handle_output(&grad)?;
                            }

                            Err(err) => {
                                if self.cfg.is_stdout || (self.output_mode == OutputMode::Gradient)
                                {
                                    writeln!(self.stdout, "\n  \x1B[31m{}\x1B[39m", err)?;
                                }
                            }
                        }
                    }

                    "svg" => {
                        let filename = &path.display().to_string();
                        let gradients =
                            parse_svg(path.into_os_string().into_string().unwrap().as_ref());

                        if (self.cfg.is_stdout || (self.output_mode == OutputMode::Gradient))
                            && gradients.is_empty()
                        {
                            writeln!(self.stdout, "{}", filename)?;
                            writeln!(self.stdout, "  \x1B[31mNo gradients.\x1B[39m")?;
                        }

                        for (grad, id) in gradients {
                            let (id, stop) = if let Some(id) = id {
                                if let Some(ref id2) = self.opt.svg_id {
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

                            if self.cfg.is_stdout || (self.output_mode == OutputMode::Gradient) {
                                writeln!(self.stdout, "{} \x1B[1m{}\x1B[0m", filename, id)?;
                            }

                            self.handle_output(&grad)?;

                            if stop {
                                break;
                            }
                        }
                    }
                    _ => continue,
                }
            }
        }

        Ok(0)
    }

    fn handle_output(&mut self, grad: &Gradient) -> io::Result<i32> {
        match self.output_mode {
            OutputMode::Gradient => self.display_gradient(grad),

            OutputMode::ColorsN => self.display_colors(&grad.colors(self.opt.take.unwrap())),

            OutputMode::ColorsSample => {
                let colors = self
                    .opt
                    .sample
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|t| grad.at(*t))
                    .collect::<Vec<_>>();

                self.display_colors(&colors)
            }
        }
    }

    fn display_gradient(&mut self, grad: &Gradient) -> io::Result<i32> {
        let (dmin, dmax) = grad.domain();
        let w2 = (self.cfg.width * 2 - 1) as f64;
        let cb_0 = &self.cfg.cb_color[0];
        let cb_1 = &self.cfg.cb_color[1];

        for y in 0..self.cfg.height {
            let mut i = 0;

            for x in 0..self.cfg.width {
                let bg_color = if self.cfg.use_solid_bg {
                    &self.cfg.background
                } else if ((x / 2) & 1) ^ (y & 1) == 0 {
                    cb_0
                } else {
                    cb_1
                };

                let col_l = grad.at(remap(i as f64, 0.0, w2, dmin, dmax));
                i += 1;

                let col_r = grad.at(remap(i as f64, 0.0, w2, dmin, dmax));
                i += 1;

                let col_l = blend(&col_l, bg_color).rgba_u8();
                let col_r = blend(&col_r, bg_color).rgba_u8();

                write!(
                    self.stdout,
                    "\x1B[38;2;{};{};{};48;2;{};{};{}m\u{258C}",
                    col_l.0, col_l.1, col_l.2, col_r.0, col_r.1, col_r.2
                )?;
            }

            writeln!(self.stdout, "\x1B[39;49m")?;
        }

        Ok(0)
    }

    fn display_colors(&mut self, colors: &[Color]) -> io::Result<i32> {
        if self.opt.array {
            let mut cols = Vec::new();

            for col in colors {
                cols.push(if self.cfg.use_solid_bg {
                    format_color(&blend(col, &self.cfg.background), self.cfg.output_format)
                } else {
                    format_color(col, self.cfg.output_format)
                });
            }

            writeln!(self.stdout, "{:?}", cols)?;
        } else if self.cfg.is_stdout {
            let mut width = self.cfg.term_width;

            for col in colors {
                let (col, bg) = if self.cfg.use_solid_bg {
                    let c = blend(col, &self.cfg.background);
                    (c.clone(), c)
                } else {
                    (col.clone(), blend(col, &Color::from_rgb(0.0, 0.0, 0.0)))
                };

                let s = format_color(&col, self.cfg.output_format);

                if width < s.len() {
                    writeln!(self.stdout)?;
                    width = self.cfg.term_width;
                }

                let (r, g, b, _) = bg.rgba_u8();

                let fg = if color_luminance(&bg) < 0.3 {
                    (255, 255, 255)
                } else {
                    (0, 0, 0)
                };

                write!(
                    self.stdout,
                    "\x1B[38;2;{};{};{};48;2;{};{};{}m{}\x1B[39;49m",
                    fg.0, fg.1, fg.2, r, g, b, &s
                )?;

                width -= s.len();

                if width >= 1 {
                    write!(self.stdout, " ")?;
                    width -= 1;
                }
            }

            writeln!(self.stdout)?;
        } else {
            for col in colors {
                if self.cfg.use_solid_bg {
                    writeln!(
                        self.stdout,
                        "{}",
                        format_color(&blend(col, &self.cfg.background), self.cfg.output_format)
                    )?;
                } else {
                    writeln!(self.stdout, "{}", format_color(col, self.cfg.output_format))?;
                }
            }
        }

        Ok(0)
    }
}

fn example_help() -> io::Result<i32> {
    fn parse_colors(colors: &[&str]) -> Vec<Color> {
        colors.iter().map(|s| parse_color(s).unwrap()).collect()
    }

    let mut stdout = io::stdout();

    let mut opt = Opt {
        preset: Some("rainbow".to_string()),
        width: Some(80),
        ..Default::default()
    };

    writeln!(stdout, "\x1B[1mEXAMPLES:\x1B[0m\n")?;

    writeln!(
        stdout,
        "\x1B[38;5;10m\u{21AA}\x1B[39m  gradient --preset rainbow"
    )?;
    let mut ga = GradientApp::new(opt.clone(), io::stdout());
    ga.run()?;

    writeln!(
        stdout,
        "\x1B[38;5;10m\u{21AA}\x1B[39m  gradient --preset turbo"
    )?;
    opt.preset = Some("turbo".to_string());
    let mut ga = GradientApp::new(opt.clone(), io::stdout());
    ga.run()?;

    writeln!(
        stdout,
        "\x1B[38;5;10m\u{21AA}\x1B[39m  gradient --custom C41189 'rgb(0,191,255)' gold 'hsv(91,88%,50%)'"
    )?;
    opt.preset = None;
    opt.custom = Some(parse_colors(&[
        "C41189",
        "rgb(0,191,255)",
        "gold",
        "hsv(91,88%,50%)",
    ]));
    let mut ga = GradientApp::new(opt.clone(), io::stdout());
    ga.run()?;

    writeln!(
        stdout,
        "\x1B[38;5;10m\u{21AA}\x1B[39m  gradient --file Test.svg Neon_Green.ggr"
    )?;

    writeln!(stdout, "Test.svg \x1B[1m#purple-gradient\x1B[0m")?;
    opt.custom = Some(parse_colors(&["4a1578", "c5a8de"]));
    let mut ga = GradientApp::new(opt.clone(), io::stdout());
    ga.run()?;

    writeln!(stdout, "Neon_Green.ggr \x1B[1mNeon Green\x1B[0m")?;
    let color = Color::from_rgb(0.0, 0.0, 0.0);
    let grad = colorgrad::parse_ggr(BufReader::new(GGR_SAMPLE.as_bytes()), &color, &color)
        .unwrap()
        .0;
    ga.display_gradient(&grad)?;

    writeln!(
        stdout,
        "\x1B[38;5;10m\u{21AA}\x1B[39m  gradient --preset viridis --take 10"
    )?;
    opt.custom = None;
    opt.preset = Some("viridis".to_string());
    opt.take = Some(10);
    let mut ga = GradientApp::new(opt, io::stdout());
    ga.run()?;

    Ok(1)
}

fn main() {
    let opt = Opt::parse();

    let mut ga = GradientApp::new(opt, io::stdout());

    match ga.run() {
        Ok(exit_code) => {
            exit(exit_code);
        }

        Err(err) if err.kind() == io::ErrorKind::BrokenPipe => {
            exit(0);
        }

        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }
}

fn parse_color(s: &str) -> Result<Color, colorgrad::ParseColorError> {
    s.parse::<Color>()
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

// Map t from range [a, b] to range [c, d]
fn remap(t: f64, a: f64, b: f64, c: f64, d: f64) -> f64 {
    (t - a) * ((d - c) / (b - a)) + c
}
