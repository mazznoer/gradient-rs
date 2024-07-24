use clap::Parser;
use colorgrad::{preset, Color, Gradient};
use std::io::{self, BufReader, IsTerminal, Write};
use std::{ffi::OsStr, fs::File, process::exit};

mod cli;
use cli::{BlendMode, Interpolation, Opt, OutputColor, PRESET_NAMES};

mod svg_gradient;
use svg_gradient::parse_svg;

mod util;

#[derive(PartialEq)]
enum OutputMode {
    Gradient,
    ColorsN,
    ColorsSample,
}

struct GradientApp {
    opt: Opt,
    stdout: io::Stdout,
    is_terminal: bool,
    output_mode: OutputMode,
    output_format: OutputColor,
    use_solid_bg: bool,
    background: Color,
    cb_color: [Color; 2],
    term_width: usize,
    width: usize,
    height: usize,
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
            [c[0].clone(), c[1].clone()]
        } else {
            [
                Color::new(0.05, 0.05, 0.05, 1.0),
                Color::new(0.20, 0.20, 0.20, 1.0),
            ]
        };

        let width = opt
            .width
            .unwrap_or_else(|| term_width.unwrap_or(80))
            .max(10)
            .min(term_width.unwrap_or(1000));

        let output_mode = if opt.take.is_some() {
            OutputMode::ColorsN
        } else if opt.sample.is_some() {
            OutputMode::ColorsSample
        } else {
            OutputMode::Gradient
        };

        let is_terminal = stdout.is_terminal();

        Self {
            output_mode,
            stdout,
            is_terminal,
            use_solid_bg: opt.background.is_some(),
            background,
            cb_color,
            term_width: term_width.unwrap_or(80),
            width,
            height: opt.height.unwrap_or(2).clamp(1, 50),
            output_format: opt.format.unwrap_or(OutputColor::Hex),
            opt,
        }
    }

    fn run(&mut self) -> io::Result<i32> {
        if self.opt.list_presets {
            self.width = self.term_width.min(80);
            self.height = 2;

            for name in &PRESET_NAMES {
                writeln!(self.stdout, "{name}")?;
                self.opt.preset = Some(name.to_string());
                self.preset_gradient()?;
            }

            return Ok(0);
        }

        if self.opt.list_named_colors {
            for (&name, &[r, g, b]) in csscolorparser::NAMED_COLORS.entries() {
                writeln!(
                    self.stdout,
                    "\x1B[48;2;{r};{g};{b}m   \x1B[49;38;2;{r};{g};{b}m #{r:02x}{g:02x}{b:02x}\x1B[39m {name}"
                )?;
            }

            return Ok(0);
        }

        if self.opt.preset.is_some() {
            return self.preset_gradient();
        }

        if self.opt.custom.is_some() || self.opt.css.is_some() {
            return self.custom_gradient();
        }

        if self.opt.file.is_some() {
            return self.file_gradient();
        }

        self.example_help()?;
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

        if let Some(ref css_gradient) = self.opt.css {
            gb.css(css_gradient);
        } else {
            gb.colors(self.opt.custom.as_ref().unwrap());

            if let Some(ref pos) = self.opt.position {
                gb.domain(pos);
            }
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
                        if self.is_terminal || (self.output_mode == OutputMode::Gradient) {
                            write!(self.stdout, "{}", &path.display())?;
                        }

                        let f = File::open(&path)?;

                        match colorgrad::GimpGradient::new(
                            BufReader::new(f),
                            &ggr_fg_color,
                            &ggr_bg_color,
                        ) {
                            Ok(grad) => {
                                if self.is_terminal || (self.output_mode == OutputMode::Gradient) {
                                    writeln!(self.stdout, " \x1B[1m{}\x1B[0m", grad.name())?;
                                }

                                self.handle_output(Box::new(grad))?;
                            }

                            Err(err) => {
                                if self.is_terminal || (self.output_mode == OutputMode::Gradient) {
                                    writeln!(self.stdout, "\n  \x1B[31m{err}\x1B[39m")?;
                                }
                            }
                        }
                    }

                    "svg" => {
                        let filename = &path.display().to_string();
                        let gradients =
                            parse_svg(path.into_os_string().into_string().unwrap().as_ref());

                        if (self.is_terminal || (self.output_mode == OutputMode::Gradient))
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

                            if self.is_terminal || (self.output_mode == OutputMode::Gradient) {
                                writeln!(self.stdout, "{filename} \x1B[1m{id}\x1B[0m")?;
                            }

                            self.handle_output(Box::new(grad))?;

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

            OutputMode::ColorsN => {
                let mut colors = grad.colors(self.opt.take.unwrap());
                if self.use_solid_bg {
                    for col in &mut colors {
                        util::blend_on(col, &self.background);
                    }
                }
                self.display_colors(&colors)
            }

            OutputMode::ColorsSample => {
                let mut colors = Vec::with_capacity(self.opt.sample.as_ref().unwrap().len());
                for pos in self.opt.sample.as_ref().unwrap().iter() {
                    let mut col = grad.at(*pos).clamp();
                    if self.use_solid_bg {
                        util::blend_on(&mut col, &self.background);
                    }
                    colors.push(col);
                }
                self.display_colors(&colors)
            }
        }
    }

    fn display_gradient(&mut self, grad: Box<dyn Gradient>) -> io::Result<i32> {
        let (dmin, dmax) = grad.domain();
        let w2 = (self.width * 2 - 1) as f32;
        let [cb_0, cb_1] = &self.cb_color;

        for y in 0..self.height {
            let mut i = 0;

            for x in 0..self.width {
                let bg_color = if self.use_solid_bg {
                    &self.background
                } else if ((x / 2) & 1) ^ (y & 1) == 0 {
                    cb_0
                } else {
                    cb_1
                };

                let col_l = grad.at(util::remap(i as f32, 0.0, w2, dmin, dmax));
                i += 1;

                let col_r = grad.at(util::remap(i as f32, 0.0, w2, dmin, dmax));
                i += 1;

                let col_l = util::blend_color(&col_l, bg_color).to_rgba8();
                let col_r = util::blend_color(&col_r, bg_color).to_rgba8();

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
            let mut cols = Vec::with_capacity(colors.len());
            for col in colors {
                cols.push(util::format_color(col, self.output_format));
            }
            writeln!(self.stdout, "{cols:?}")?;
            return Ok(0);
        }

        if self.is_terminal {
            if self.output_format != OutputColor::Hex {
                for col in colors {
                    writeln!(
                        self.stdout,
                        "{} {}",
                        util::fmt_color(col, &self.cb_color, 7),
                        util::format_color(col, self.output_format)
                    )?;
                }
                return Ok(0);
            }

            let mut buff0 = "".to_string();
            let mut buff1 = "".to_string();
            let last = colors.len() - 1;
            let mut w = 0;

            for (i, col) in colors.iter().enumerate() {
                let hex = util::format_color(col, self.output_format);
                let wc = hex.len();
                buff0.push_str(&util::fmt_color(col, &self.cb_color, wc));
                buff1.push_str(&hex);
                w += wc;
                if w < self.term_width {
                    buff0.push(' ');
                    buff1.push(' ');
                    w += 1;
                }
                let nwc = if i == last {
                    0
                } else {
                    util::format_color(&colors[i + 1], self.output_format).len()
                };

                if w + nwc > self.term_width || i == last {
                    writeln!(self.stdout, "{}\n{}", buff0, buff1)?;
                    buff0.clear();
                    buff1.clear();
                    w = 0;
                }
            }
            return Ok(0);
        }

        for col in colors {
            writeln!(
                self.stdout,
                "{}",
                util::format_color(col, self.output_format)
            )?;
        }
        Ok(0)
    }

    fn example_help(&mut self) -> io::Result<i32> {
        fn parse_colors(colors: &[&str]) -> Vec<Color> {
            colors
                .iter()
                .map(|s| Color::from_html(s).unwrap())
                .collect()
        }

        fn bold(s: &str) -> String {
            format!("\x1B[1m{s}\x1B[0m")
        }

        let prompt = "\u{21AA} ";
        self.width = self.term_width.min(80);
        self.height = 2;

        writeln!(
            self.stdout,
            "{} Specify gradient using --preset, --custom, --css or --file\n",
            bold("INFO:")
        )?;
        writeln!(self.stdout, "{}", bold("EXAMPLES:"))?;
        writeln!(self.stdout, "{prompt} gradient --preset rainbow")?;
        self.opt.preset = Some("rainbow".to_string());
        self.preset_gradient()?;

        writeln!(
            self.stdout,
            "{prompt} gradient --custom C41189 'rgb(0,191,255)' gold 'hsv(91,88%,50%)'"
        )?;
        self.opt.preset = None;
        self.opt.custom = Some(parse_colors(&[
            "C41189",
            "rgb(0,191,255)",
            "gold",
            "hsv(91,88%,50%)",
        ]));
        self.custom_gradient()?;

        writeln!(self.stdout, "{prompt} gradient --css 'white, 25%, blue'")?;
        self.opt.custom = None;
        self.opt.css = Some("white, 25%, blue".to_string());
        self.custom_gradient()?;

        writeln!(
            self.stdout,
            "{prompt} gradient --file Test.svg Neon_Green.ggr"
        )?;
        writeln!(self.stdout, "Test.svg {}", bold("#purple-gradient"))?;
        self.opt.css = None;
        self.opt.custom = Some(parse_colors(&["4a1578", "c5a8de"]));
        self.custom_gradient()?;

        writeln!(self.stdout, "Neon_Green.ggr {}", bold("Neon Green"))?;
        const GGR_SAMPLE: &str = include_str!("../data/Neon_Green.ggr");
        let color = Color::default();
        let grad =
            colorgrad::GimpGradient::new(BufReader::new(GGR_SAMPLE.as_bytes()), &color, &color)
                .unwrap();
        self.display_gradient(Box::new(grad))?;

        writeln!(self.stdout, "{prompt} gradient --preset viridis --take 10")?;
        self.opt.custom = None;
        self.opt.preset = Some("viridis".to_string());
        self.opt.take = Some(10);
        self.output_mode = OutputMode::ColorsN;
        self.preset_gradient()?;

        Ok(0)
    }
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
