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

pub fn fmt_color(col: &Color, cb: &[Color; 2], width: usize) -> String {
    let mut ss = "".to_string();
    for i in 0..width {
        let ch = if (i & 1) == 0 { "\u{2580}" } else { "\u{2584}" };
        let cl = blend_color(col, &cb[0]).to_rgba8();
        let cr = blend_color(col, &cb[1]).to_rgba8();
        ss.push_str(&format!(
            "\x1B[38;2;{};{};{};48;2;{};{};{}m{}",
            cl[0], cl[1], cl[2], cr[0], cr[1], cr[2], ch
        ));
    }
    ss.push_str("\x1B[39;49m");
    ss
}

fn format_alpha(a: f32) -> String {
    if a >= 1.0 {
        return "".into();
    }
    let s = format!(",{:.2}%", a * 100.0);
    if s.starts_with(",100") {
        return "".into();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(format_alpha(0.0), ",0.00%");
        assert_eq!(format_alpha(0.5), ",50.00%");
        assert_eq!(format_alpha(1.0), "");
        assert_eq!(format_alpha(1.2), "");

        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        assert_eq!(format_color(&red, OutputColor::Hex), "#ff0000");
        assert_eq!(
            format_color(&red, OutputColor::Rgb),
            "rgb(100.00%,0.00%,0.00%)"
        );
        assert_eq!(format_color(&red, OutputColor::Rgb255), "rgb(255,0,0)");
        assert_eq!(
            format_color(&red, OutputColor::Hsl),
            "hsl(0.00,100.00%,50.00%)"
        );
        assert_eq!(
            format_color(&red, OutputColor::Hsv),
            "hsv(0.00,100.00%,100.00%)"
        );
        assert_eq!(
            format_color(&red, OutputColor::Hwb),
            "hwb(0.00,0.00%,0.00%)"
        );
    }
}
