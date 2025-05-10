use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, BufReader, IsTerminal, Read, Write};
use std::process::exit;

use colorgrad::{Color, GimpGradient, Gradient};

mod cli;
use cli::{BlendMode, Interpolation, Opt, OutputColor, PRESET_NAMES};

mod svg_gradient;

mod util;
use util::bold;

#[derive(PartialEq)]
enum OutputMode {
    Gradient,
    ColorsN,
    ColorsSample,
}

struct GradientApp<'a> {
    opt: Opt,
    stdout: io::StdoutLock<'a>,
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

impl GradientApp<'_> {
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
            stdout: stdout.lock(),
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

            for name in PRESET_NAMES {
                writeln!(self.stdout, "{name}")?;
                self.opt.preset = Some(name.into());
                self.preset_gradient()?;
            }

            return Ok(0);
        }

        if self.opt.named_colors {
            let mut colors: Vec<_> = csscolorparser::NAMED_COLORS.entries().collect();
            colors.sort_by(|a, b| a.0.cmp(b.0));

            for (&name, &[r, g, b]) in &colors {
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
        use colorgrad::preset::*;

        fn cubehelix() -> CubehelixGradient {
            cubehelix_default()
        }

        macro_rules! presets {
            ($($name:ident),*) => {
                match self
                .opt
                .preset
                .as_ref()
                .unwrap()
                .to_lowercase()
                .replace('-', "_")
                .as_ref()
                {
                    $(stringify!($name) => self.handle_output(&$name())?,)*
                    _ => {
                        writeln!(io::stderr(), "Error: Invalid preset gradient name. Use -l flag to list all preset gradient names.")?;
                        return Ok(1);
                    }
                }
            }
        }

        presets!(
            blues, br_bg, bu_gn, bu_pu, cividis, cool, cubehelix, gn_bu, greens, greys, inferno,
            magma, or_rd, oranges, pi_yg, plasma, pr_gn, pu_bu, pu_bu_gn, pu_or, pu_rd, purples,
            rainbow, rd_bu, rd_gy, rd_pu, rd_yl_bu, rd_yl_gn, reds, sinebow, spectral, turbo,
            viridis, warm, yl_gn, yl_gn_bu, yl_or_br, yl_or_rd
        );

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

        match self.opt.interpolation {
            Some(Interpolation::Linear) => match gb.build::<colorgrad::LinearGradient>() {
                Ok(g) => self.handle_output(&g)?,
                Err(e) => {
                    writeln!(io::stderr(), "Custom gradient error: {e}")?;
                    return Ok(1);
                }
            },
            Some(Interpolation::Basis) => match gb.build::<colorgrad::BasisGradient>() {
                Ok(g) => self.handle_output(&g)?,
                Err(e) => {
                    writeln!(io::stderr(), "Custom gradient error: {e}")?;
                    return Ok(1);
                }
            },
            _ => match gb.build::<colorgrad::CatmullRomGradient>() {
                Ok(g) => self.handle_output(&g)?,
                Err(e) => {
                    writeln!(io::stderr(), "Custom gradient error: {e}")?;
                    return Ok(1);
                }
            },
        };

        Ok(0)
    }

    fn file_gradient(&mut self) -> io::Result<i32> {
        use colorgrad::{BasisGradient, CatmullRomGradient, LinearGradient};

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

        let show_info = self.is_terminal || self.output_mode == OutputMode::Gradient;
        let mut status = 0;

        for path in self.opt.file.as_ref().unwrap().clone() {
            if !path.exists() {
                eprintln!("{}: file not found.", &path.display());
                status = 1;
                continue;
            }

            let Some(ext) = path.extension().and_then(OsStr::to_str) else {
                eprintln!("{}: file format not supported.", &path.display());
                status = 1;
                continue;
            };

            let ext = ext.to_lowercase();

            if &ext == "ggr" {
                match GimpGradient::new(
                    BufReader::new(File::open(&path)?),
                    &ggr_fg_color,
                    &ggr_bg_color,
                ) {
                    Ok(grad) => {
                        if show_info {
                            writeln!(self.stdout, "{} {}", &path.display(), bold(grad.name()))?;
                        }
                        self.handle_output(&grad)?;
                    }
                    Err(_) => {
                        eprintln!("{} (invalid GIMP gradient)", &path.display());
                        status = 1;
                        continue;
                    }
                }
            } else if &ext == "svg" {
                let mut file = File::open(&path)?;
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                let svg_grads = svg_gradient::parse_svg(&content, self.opt.svg_id.as_deref());

                let cmode = match self.opt.blend_mode {
                    Some(BlendMode::Rgb) => colorgrad::BlendMode::Rgb,
                    Some(BlendMode::LinearRgb) => colorgrad::BlendMode::LinearRgb,
                    Some(BlendMode::Lab) => colorgrad::BlendMode::Lab,
                    _ => colorgrad::BlendMode::Oklab,
                };
                let mut valid = 0;
                let mut invalid = 0;

                for mut sg in svg_grads {
                    assert_eq!(sg.colors.len(), sg.pos.len());

                    let id = sg
                        .id
                        .as_ref()
                        .map(|s| format!("#{s}"))
                        .unwrap_or("[without id]".into());

                    let Some(mut gb) = sg.gradient_builder() else {
                        eprintln!("{} {} (invalid gradient)", &path.display(), bold(&id));
                        status = 1;
                        invalid += 1;
                        continue;
                    };

                    if show_info {
                        writeln!(self.stdout, "{} {}", &path.display(), bold(&id))?;
                    }

                    gb.mode(cmode);

                    match self.opt.interpolation {
                        Some(Interpolation::Linear) => {
                            let g: LinearGradient = gb.build().unwrap();
                            self.handle_output(&g)?;
                        }
                        Some(Interpolation::Basis) => {
                            let g: BasisGradient = gb.build().unwrap();
                            self.handle_output(&g)?;
                        }
                        _ => {
                            let g: CatmullRomGradient = gb.build().unwrap();
                            self.handle_output(&g)?;
                        }
                    }
                    valid += 1;
                }

                if valid == 0 && invalid == 0 {
                    if self.opt.svg_id.is_some() {
                        eprintln!("{} -- (nothing matched)", &path.display(),);
                    } else {
                        eprintln!("{} -- (no gradients found)", &path.display());
                    }
                    status = 1;
                }
            } else {
                eprintln!("{}: file format not supported.", &path.display());
                status = 1;
            }
        }

        Ok(status)
    }

    fn handle_output(&mut self, grad: &dyn Gradient) -> io::Result<i32> {
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
                let colors: Vec<_> = self
                    .opt
                    .sample
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|t| {
                        let mut c = grad.at(*t).clamp();
                        if self.use_solid_bg {
                            util::blend_on(&mut c, &self.background);
                        }
                        c
                    })
                    .collect();
                self.display_colors(&colors)
            }
        }
    }

    fn display_gradient(&mut self, grad: &dyn Gradient) -> io::Result<i32> {
        let colors = grad.colors(self.width * 2);
        let mut out = io::BufWriter::new(&mut self.stdout);

        for y in 0..self.height {
            for (x, cols) in colors.chunks_exact(2).enumerate() {
                let bg_color = if self.use_solid_bg {
                    &self.background
                } else if ((x / 2) & 1) ^ (y & 1) == 0 {
                    &self.cb_color[0]
                } else {
                    &self.cb_color[1]
                };

                let [a, b, c, _] = util::blend_color(&cols[0], bg_color).to_rgba8();
                let [d, e, f, _] = util::blend_color(&cols[1], bg_color).to_rgba8();

                write!(out, "\x1B[38;2;{a};{b};{c};48;2;{d};{e};{f}m\u{258C}",)?;
            }

            writeln!(out, "\x1B[39;49m")?;
        }

        out.flush()?;
        Ok(0)
    }

    fn display_colors(&mut self, colors: &[Color]) -> io::Result<i32> {
        let mut out = io::BufWriter::new(&mut self.stdout);

        if self.opt.array {
            let f = self.output_format;
            let cols: Vec<_> = colors.iter().map(|c| util::format_color(c, f)).collect();
            writeln!(out, "{cols:?}")?;
            out.flush()?;
            return Ok(0);
        }

        if self.is_terminal {
            if self.output_format != OutputColor::Hex {
                for col in colors {
                    writeln!(
                        out,
                        "{} {}",
                        util::color_to_ansi(col, &self.cb_color, 7),
                        util::format_color(col, self.output_format)
                    )?;
                }
                out.flush()?;
                return Ok(0);
            }

            let mut buff0 = "".to_string();
            let mut buff1 = "".to_string();
            let last = colors.len() - 1;
            let mut w = 0;

            for (i, col) in colors.iter().enumerate() {
                let hex = util::format_color(col, self.output_format);
                let wc = hex.len();
                buff0.push_str(&util::color_to_ansi(col, &self.cb_color, wc));
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
                    writeln!(out, "{}\n{}", buff0, buff1)?;
                    buff0.clear();
                    buff1.clear();
                    w = 0;
                }
            }
            out.flush()?;
            return Ok(0);
        }

        for col in colors {
            writeln!(out, "{}", util::format_color(col, self.output_format))?;
        }
        out.flush()?;
        Ok(0)
    }

    fn example_help(&mut self) -> io::Result<i32> {
        fn parse_colors(colors: &[&str]) -> Vec<Color> {
            colors
                .iter()
                .map(|s| Color::from_html(s).unwrap())
                .collect()
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
        self.opt.preset = Some("rainbow".into());
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
        self.opt.css = Some("white, 25%, blue".into());
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
        let br = BufReader::new(GGR_SAMPLE.as_bytes());
        let grad = GimpGradient::new(br, &color, &color).unwrap();
        self.display_gradient(&grad)?;

        writeln!(self.stdout, "{prompt} gradient --preset viridis --take 10")?;
        self.opt.custom = None;
        self.opt.preset = Some("viridis".into());
        self.opt.take = Some(10);
        self.output_mode = OutputMode::ColorsN;
        self.preset_gradient()?;

        Ok(0)
    }
}

fn main() {
    let opt = match cli::parse_args() {
        Ok(opt) => opt,
        Err(err) => {
            eprintln!("Error: {err}.");
            exit(1);
        }
    };

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
