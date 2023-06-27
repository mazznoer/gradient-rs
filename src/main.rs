use clap::Parser;
use colorgrad::{preset, Color, Gradient};
use std::io::{self, BufReader, Write};
use std::{ffi::OsStr, fs::File, process::exit};

mod cli;
use cli::{BlendMode, Interpolation, Opt, OutputColor};

mod svg_gradient;
use svg_gradient::parse_svg;

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
            Color::new(0.0, 0.0, 0.0, 1.0)
        };

        let cb_color = if let Some(ref c) = opt.cb_color {
            c.clone()
        } else {
            vec![
                Color::new(0.05, 0.05, 0.05, 1.0),
                Color::new(0.20, 0.20, 0.20, 1.0),
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
                writeln!(self.stdout, "{name}")?;
                let opt = Opt {
                    preset: Some(name.to_string()),
                    width: Some(80),
                    height: Some(1),
                    ..Default::default()
                };
                let mut ga = GradientApp::new(opt, io::stdout());
                ga.run()?;
            }

            return Ok(0);
        }

        if self.opt.preset.is_some() {
            return self.preset_gradient();
        }

        if self.opt.custom.is_some() {
            return self.custom_gradient();
        }

        if self.opt.file.is_some() {
            return self.file_gradient();
        }

        writeln!(
            self.stdout,
            "\x1B[38;5;9merror:\x1B[39m Specify gradient using -p / --preset, -c / --custom or -f / --file\n"
        )?;

        example_help()?;
        Ok(1)
    }

    fn preset_gradient(&mut self) -> io::Result<i32> {
        let grad: Box<dyn Gradient> = match self
            .opt
            .preset
            .as_ref()
            .unwrap()
            .to_lowercase()
            .replace('-', "_")
            .as_ref()
        {
            "blues" => Box::new(preset::blues()),
            "br_bg" => Box::new(preset::br_bg()),
            "bu_gn" => Box::new(preset::bu_gn()),
            "bu_pu" => Box::new(preset::bu_pu()),
            "cividis" => Box::new(preset::cividis()),
            "cool" => Box::new(preset::cool()),
            "cubehelix" => Box::new(preset::cubehelix_default()),
            "gn_bu" => Box::new(preset::gn_bu()),
            "greens" => Box::new(preset::greens()),
            "greys" => Box::new(preset::greys()),
            "inferno" => Box::new(preset::inferno()),
            "magma" => Box::new(preset::magma()),
            "or_rd" => Box::new(preset::or_rd()),
            "oranges" => Box::new(preset::oranges()),
            "pi_yg" => Box::new(preset::pi_yg()),
            "plasma" => Box::new(preset::plasma()),
            "pr_gn" => Box::new(preset::pr_gn()),
            "pu_bu" => Box::new(preset::pu_bu()),
            "pu_bu_gn" => Box::new(preset::pu_bu_gn()),
            "pu_or" => Box::new(preset::pu_or()),
            "pu_rd" => Box::new(preset::pu_rd()),
            "purples" => Box::new(preset::purples()),
            "rainbow" => Box::new(preset::rainbow()),
            "rd_bu" => Box::new(preset::rd_bu()),
            "rd_gy" => Box::new(preset::rd_gy()),
            "rd_pu" => Box::new(preset::rd_pu()),
            "rd_yl_bu" => Box::new(preset::rd_yl_bu()),
            "rd_yl_gn" => Box::new(preset::rd_yl_gn()),
            "reds" => Box::new(preset::reds()),
            "sinebow" => Box::new(preset::sinebow()),
            "spectral" => Box::new(preset::spectral()),
            "turbo" => Box::new(preset::turbo()),
            "viridis" => Box::new(preset::viridis()),
            "warm" => Box::new(preset::warm()),
            "yl_gn" => Box::new(preset::yl_gn()),
            "yl_gn_bu" => Box::new(preset::yl_gn_bu()),
            "yl_or_br" => Box::new(preset::yl_or_br()),
            "yl_or_rd" => Box::new(preset::yl_or_rd()),
            _ => {
                writeln!(io::stderr(), "Error: Invalid preset gradient name. Use -l flag to list all preset gradient names.")?;
                return Ok(1);
            }
        };

        self.handle_output(grad)?;
        Ok(0)
    }

    fn custom_gradient(&mut self) -> io::Result<i32> {
        let mut gb = colorgrad::GradientBuilder::new();

        gb.colors(self.opt.custom.as_ref().unwrap());

        if let Some(ref pos) = self.opt.position {
            gb.domain(pos);
        }

        gb.mode(match self.opt.blend_mode {
            Some(BlendMode::Rgb) => colorgrad::BlendMode::Rgb,
            Some(BlendMode::LinearRgb) => colorgrad::BlendMode::LinearRgb,
            Some(BlendMode::Lab) => colorgrad::BlendMode::Lab,
            _ => colorgrad::BlendMode::Oklab,
        });

        let grad: Box<dyn Gradient> = match self.opt.interpolation {
            Some(Interpolation::Linear) => match gb.build::<colorgrad::LinearGradient>() {
                Ok(g) => Box::new(g),
                Err(e) => {
                    writeln!(io::stderr(), "Custom gradient error: {e}")?;
                    return Ok(1);
                }
            },
            Some(Interpolation::Basis) => match gb.build::<colorgrad::BasisGradient>() {
                Ok(g) => Box::new(g),
                Err(e) => {
                    writeln!(io::stderr(), "Custom gradient error: {e}")?;
                    return Ok(1);
                }
            },
            _ => match gb.build::<colorgrad::CatmullRomGradient>() {
                Ok(g) => Box::new(g),
                Err(e) => {
                    writeln!(io::stderr(), "Custom gradient error: {e}")?;
                    return Ok(1);
                }
            },
        };

        self.handle_output(grad)?;
        Ok(0)
    }

    fn file_gradient(&mut self) -> io::Result<i32> {
        let ggr_bg_color = if let Some(ref c) = self.opt.ggr_bg {
            c.clone()
        } else {
            Color::new(1.0, 1.0, 1.0, 1.0)
        };

        let ggr_fg_color = if let Some(ref c) = self.opt.ggr_fg {
            c.clone()
        } else {
            Color::new(0.0, 0.0, 0.0, 1.0)
        };

        let mut status = 0;

        for path in self.opt.file.as_ref().unwrap().clone() {
            if !path.exists() {
                write!(
                    self.stdout,
                    "{}\n  \x1B[31mFile not found\x1B[39m\n",
                    &path.display()
                )?;
                status = 1;
                continue;
            }

            if let Some(ext) = path.extension().and_then(OsStr::to_str) {
                match ext.to_lowercase().as_ref() {
                    "ggr" => {
                        if self.cfg.is_stdout || (self.output_mode == OutputMode::Gradient) {
                            write!(self.stdout, "{}", &path.display())?;
                        }

                        let f = File::open(&path).unwrap();

                        match colorgrad::GimpGradient::new(
                            BufReader::new(f),
                            &ggr_fg_color,
                            &ggr_bg_color,
                        ) {
                            Ok(grad) => {
                                if self.cfg.is_stdout || (self.output_mode == OutputMode::Gradient)
                                {
                                    writeln!(self.stdout, " \x1B[1m{}\x1B[0m", grad.name())?;
                                }

                                self.handle_output(Box::new(grad) as Box<dyn Gradient>)?;
                            }

                            Err(err) => {
                                if self.cfg.is_stdout || (self.output_mode == OutputMode::Gradient)
                                {
                                    writeln!(self.stdout, "\n  \x1B[31m{err}\x1B[39m")?;
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
                            writeln!(self.stdout, "{filename}")?;
                            writeln!(self.stdout, "  \x1B[31mNo gradients.\x1B[39m")?;
                        }

                        for (grad, id) in gradients {
                            let (id, stop) = if let Some(id) = id {
                                if let Some(ref id2) = self.opt.svg_id {
                                    if &id == id2 {
                                        (format!("#{id}"), true)
                                    } else {
                                        continue;
                                    }
                                } else {
                                    (format!("#{id}"), false)
                                }
                            } else {
                                ("".to_string(), false)
                            };

                            if self.cfg.is_stdout || (self.output_mode == OutputMode::Gradient) {
                                writeln!(self.stdout, "{filename} \x1B[1m{id}\x1B[0m")?;
                            }

                            self.handle_output(Box::new(grad) as Box<dyn Gradient>)?;

                            if stop {
                                break;
                            }
                        }
                    }
                    _ => continue,
                }
            }
        }

        Ok(status)
    }

    fn handle_output(&mut self, grad: Box<dyn Gradient>) -> io::Result<i32> {
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

    fn display_gradient(&mut self, grad: Box<dyn Gradient>) -> io::Result<i32> {
        let (dmin, dmax) = grad.domain();
        let w2 = (self.cfg.width * 2 - 1) as f32;
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

                let col_l = grad.at(remap(i as f32, 0.0, w2, dmin, dmax));
                i += 1;

                let col_r = grad.at(remap(i as f32, 0.0, w2, dmin, dmax));
                i += 1;

                let col_l = blend_color(&col_l, bg_color).to_rgba8();
                let col_r = blend_color(&col_r, bg_color).to_rgba8();

                write!(
                    self.stdout,
                    "\x1B[38;2;{};{};{};48;2;{};{};{}m\u{258C}",
                    col_l[0], col_l[1], col_l[2], col_r[0], col_r[1], col_r[2]
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
                    format_color(
                        &blend_color(col, &self.cfg.background),
                        self.cfg.output_format,
                    )
                } else {
                    format_color(col, self.cfg.output_format)
                });
            }

            writeln!(self.stdout, "{cols:?}")?;
        } else if self.cfg.is_stdout {
            let mut width = self.cfg.term_width;

            for col in colors {
                let (col, bg) = if self.cfg.use_solid_bg {
                    let c = blend_color(col, &self.cfg.background);
                    (c.clone(), c)
                } else {
                    (
                        col.clone(),
                        blend_color(col, &Color::new(0.0, 0.0, 0.0, 1.0)),
                    )
                };

                let s = format_color(&col, self.cfg.output_format);

                if width < s.len() {
                    writeln!(self.stdout)?;
                    width = self.cfg.term_width;
                }

                let [r, g, b, _] = bg.to_rgba8();

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
                        format_color(
                            &blend_color(col, &self.cfg.background),
                            self.cfg.output_format
                        )
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
        colors
            .iter()
            .map(|s| Color::from_html(s).unwrap())
            .collect()
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
    let color = Color::new(0.0, 0.0, 0.0, 1.0);
    let grad = colorgrad::GimpGradient::new(BufReader::new(GGR_SAMPLE.as_bytes()), &color, &color)
        .unwrap();
    ga.display_gradient(Box::new(grad) as Box<dyn Gradient>)?;

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
            eprintln!("{err}");
            exit(1);
        }
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Opt::command().debug_assert()
}

fn blend_color(fg: &Color, bg: &Color) -> Color {
    Color::new(
        ((1.0 - fg.a) * bg.r) + (fg.a * fg.r),
        ((1.0 - fg.a) * bg.g) + (fg.a * fg.g),
        ((1.0 - fg.a) * bg.b) + (fg.a * fg.b),
        1.0,
    )
}

// Reference http://www.w3.org/TR/2008/REC-WCAG20-20081211/#relativeluminancedef
fn color_luminance(col: &Color) -> f32 {
    fn lum(t: f32) -> f32 {
        if t <= 0.03928 {
            t / 12.92
        } else {
            ((t + 0.055) / 1.055).powf(2.4)
        }
    }

    0.2126 * lum(col.r) + 0.7152 * lum(col.g) + 0.0722 * lum(col.b)
}

fn format_alpha(a: f32) -> String {
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
            format!(
                "rgb({:.2}%,{:.2}%,{:.2}%{})",
                col.r * 100.0,
                col.g * 100.0,
                col.b * 100.0,
                format_alpha(col.a)
            )
        }

        OutputColor::Rgb255 => {
            let [r, g, b, _] = col.to_rgba8();
            format!("rgb({r},{g},{b}{})", format_alpha(col.a))
        }

        OutputColor::Hsl => {
            let [h, s, l, a] = col.to_hsla();
            format!(
                "hsl({:.2},{:.2}%,{:.2}%{})",
                h,
                s * 100.0,
                l * 100.0,
                format_alpha(a)
            )
        }

        OutputColor::Hsv => {
            let [h, s, v, a] = col.to_hsva();
            format!(
                "hsv({:.2},{:.2}%,{:.2}%{})",
                h,
                s * 100.0,
                v * 100.0,
                format_alpha(a)
            )
        }

        OutputColor::Hwb => {
            let [h, w, b, a] = col.to_hwba();
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
fn remap(t: f32, a: f32, b: f32, c: f32, d: f32) -> f32 {
    (t - a) * ((d - c) / (b - a)) + c
}
