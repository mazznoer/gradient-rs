use crate::{Color, OutputColor};

pub fn blend_color(fg: &Color, bg: &Color) -> Color {
    Color::new(
        ((1.0 - fg.a) * bg.r) + (fg.a * fg.r),
        ((1.0 - fg.a) * bg.g) + (fg.a * fg.g),
        ((1.0 - fg.a) * bg.b) + (fg.a * fg.b),
        1.0,
    )
}

pub fn blend_on(fg: &mut Color, bg: &Color) {
    fg.r = ((1.0 - fg.a) * bg.r) + (fg.a * fg.r);
    fg.g = ((1.0 - fg.a) * bg.g) + (fg.a * fg.g);
    fg.b = ((1.0 - fg.a) * bg.b) + (fg.a * fg.b);
    fg.a = 1.0;
}

pub fn color_to_ansi(col: &Color, cb: &[Color; 2], width: usize) -> String {
    let mut ss = "".to_string();
    for i in 0..width {
        let chr = if (i & 1) == 0 { "\u{2580}" } else { "\u{2584}" };
        let [a, b, c, _] = blend_color(col, &cb[0]).to_rgba8();
        let [d, e, f, _] = blend_color(col, &cb[1]).to_rgba8();
        ss.push_str(&format!("\x1B[38;2;{a};{b};{c};48;2;{d};{e};{f}m{chr}"));
    }
    ss.push_str("\x1B[39;49m");
    ss
}

pub fn bold(s: &str) -> String {
    format!("\x1B[1m{s}\x1B[0m")
}

fn fmt_float(t: f32, precision: usize) -> String {
    let s = format!("{t:.precision$}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn fmt_alpha(alpha: f32) -> String {
    if alpha < 1.0 {
        format!(" / {}%", (alpha.max(0.0) * 100.0 + 0.5).floor())
    } else {
        "".into()
    }
}

fn to_hsv_str(col: &Color) -> String {
    let [h, s, v, alpha] = col.to_hsva();
    let h = if h.is_nan() {
        "none".into()
    } else {
        fmt_float(h, 2)
    };
    let s = (s * 100.0 + 0.5).floor();
    let v = (v * 100.0 + 0.5).floor();
    format!("hsv({h} {s}% {v}%{})", fmt_alpha(alpha))
}

pub fn format_color(col: &Color, format: OutputColor) -> String {
    match format {
        OutputColor::Hex => col.to_css_hex(),
        OutputColor::Rgb => col.to_css_rgb(),
        OutputColor::Hsl => col.to_css_hsl(),
        OutputColor::Hwb => col.to_css_hwb(),
        OutputColor::Hsv => to_hsv_str(col),
        OutputColor::Lab => col.to_css_lab(),
        OutputColor::Lch => col.to_css_lch(),
        OutputColor::Oklab => col.to_css_oklab(),
        OutputColor::Oklch => col.to_css_oklch(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(fmt_alpha(0.0), " / 0%");
        assert_eq!(fmt_alpha(0.5), " / 50%");
        assert_eq!(fmt_alpha(1.0), "");
        assert_eq!(fmt_alpha(1.2), "");

        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        assert_eq!(format_color(&red, OutputColor::Hex), "#ff0000");
        assert_eq!(format_color(&red, OutputColor::Rgb), "rgb(255 0 0)");
        assert_eq!(format_color(&red, OutputColor::Hsl), "hsl(0 100% 50%)");
        assert_eq!(format_color(&red, OutputColor::Hsv), "hsv(0 100% 100%)");
        assert_eq!(format_color(&red, OutputColor::Hwb), "hwb(0 0% 0%)");
    }
}
