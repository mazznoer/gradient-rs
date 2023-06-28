use crate::{Color, OutputColor};

pub fn blend_color(fg: &Color, bg: &Color) -> Color {
    Color::new(
        ((1.0 - fg.a) * bg.r) + (fg.a * fg.r),
        ((1.0 - fg.a) * bg.g) + (fg.a * fg.g),
        ((1.0 - fg.a) * bg.b) + (fg.a * fg.b),
        1.0,
    )
}

// Reference http://www.w3.org/TR/2008/REC-WCAG20-20081211/#relativeluminancedef
pub fn color_luminance(col: &Color) -> f32 {
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

pub fn format_color(col: &Color, format: OutputColor) -> String {
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
pub fn remap(t: f32, a: f32, b: f32, c: f32, d: f32) -> f32 {
    (t - a) * ((d - c) / (b - a)) + c
}
