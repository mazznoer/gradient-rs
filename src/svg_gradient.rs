use colorgrad::{Color, GradientBuilder, LinearGradient};
use svg::node::element::tag as svg_tag;
use svg::parser::Event;

fn parse_percent_or_float(s: &str) -> Option<f32> {
    if let Some(s) = s.strip_suffix('%') {
        if let Ok(t) = s.parse::<f32>() {
            return Some(t / 100.0);
        }
        return None;
    }
    s.parse::<f32>().ok()
}

// returns (color, opacity)
fn parse_styles(s: &str) -> (Option<&str>, Option<&str>) {
    let mut val = (None, None);

    for x in s.split(';') {
        let d = x.split(':').collect::<Vec<&str>>();

        if d.len() == 2 {
            match d[0].trim().to_lowercase().as_ref() {
                "stop-color" => val.0 = Some(d[1]),
                "stop-opacity" => val.1 = Some(d[1]),
                _ => {}
            }
        }
    }

    val
}

#[derive(Debug)]
struct SvgGradient {
    id: Option<String>,
    colors: Vec<Color>,
    pos: Vec<f32>,
}

fn parse_svg(path: &str) -> Vec<SvgGradient> {
    let mut res = Vec::new();
    let mut index = 0;
    let mut prev_pos = f32::NEG_INFINITY;
    let mut content = String::new();

    for event in svg::open(path, &mut content).unwrap() {
        match event {
            Event::Tag(svg_tag::LinearGradient, t, attributes)
            | Event::Tag(svg_tag::RadialGradient, t, attributes) => match t {
                svg_tag::Type::Start => {
                    let id = attributes.get("id").map(|v| v.to_string());

                    res.push(SvgGradient {
                        id,
                        colors: Vec::new(),
                        pos: Vec::new(),
                    });
                }
                svg_tag::Type::End => {
                    index += 1;
                    prev_pos = f32::NEG_INFINITY;
                }
                svg_tag::Type::Empty => {}
            },
            Event::Tag(svg_tag::Stop, _, attributes) => {
                if res.is_empty() {
                    continue;
                }

                let mut color: Option<&str> = None;
                let mut opacity: Option<&str> = None;

                if let Some(c) = attributes.get("stop-color") {
                    color = Some(c);
                }

                if let Some(c) = attributes.get("stop-opacity") {
                    opacity = Some(c);
                }

                if let Some(styles) = attributes.get("style") {
                    let (col, opac) = parse_styles(styles);

                    if let Some(c) = col {
                        color = Some(c);
                    }

                    if let Some(o) = opac {
                        opacity = Some(o);
                    }
                }

                let color = if let Some(col) = color {
                    col.parse::<Color>().ok()
                } else {
                    None
                };

                let opacity = if let Some(op) = opacity {
                    parse_percent_or_float(op)
                } else {
                    None
                };

                let offset = if let Some(pos) = attributes.get("offset") {
                    parse_percent_or_float(pos)
                } else {
                    None
                };

                let color = color.unwrap_or_else(|| Color::new(0.0, 0.0, 0.0, 1.0));

                let offset = offset.unwrap_or(prev_pos);

                let color = if let Some(opacity) = opacity {
                    Color::new(color.r, color.g, color.b, opacity.clamp(0.0, 1.0))
                } else {
                    color
                };

                prev_pos = if offset.is_finite() {
                    offset.max(prev_pos)
                } else {
                    0.0
                };

                res[index].colors.push(color);
                res[index].pos.push(prev_pos);
            }
            _ => {}
        }
    }

    res
}

fn to_gradients(data: Vec<SvgGradient>) -> Vec<(LinearGradient, Option<String>)> {
    let mut gradients = Vec::new();

    for mut g in data {
        if g.colors.is_empty() {
            continue;
        }

        if g.pos[0] > 0.0 {
            g.pos.insert(0, 0.0);
            g.colors.insert(0, g.colors[0].clone());
        }

        if g.pos.last().unwrap() < &1.0 {
            g.pos.push(1.0);
            g.colors.push(g.colors.last().unwrap().clone());
        }

        let grad = GradientBuilder::new()
            .colors(&g.colors)
            .domain(&g.pos)
            .build::<LinearGradient>();

        match grad {
            Ok(grad) => gradients.push((grad, g.id)),
            Err(e) => eprintln!("{e}"),
        }
    }

    gradients
}

pub(crate) fn parse(path: &str) -> Vec<(LinearGradient, Option<String>)> {
    to_gradients(parse_svg(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn colors2hex(colors: &[Color]) -> Vec<String> {
        colors.iter().map(|c| c.to_hex_string()).collect()
    }

    #[test]
    fn utils() {
        assert_eq!(parse_percent_or_float("50%"), Some(0.5));
        assert_eq!(parse_percent_or_float("100%"), Some(1.0));
        assert_eq!(parse_percent_or_float("1"), Some(1.0));
        assert_eq!(parse_percent_or_float("0.73"), Some(0.73));

        assert_eq!(parse_percent_or_float(""), None);
        assert_eq!(parse_percent_or_float("16g7"), None);

        assert_eq!(
            parse_styles("stop-color:#ff0; stop-opacity:0.5"),
            (Some("#ff0"), Some("0.5"))
        );
        assert_eq!(parse_styles("stop-color:skyblue"), (Some("skyblue"), None));
        assert_eq!(parse_styles("stop-opacity:50%"), (None, Some("50%")));
        assert_eq!(parse_styles(""), (None, None));
    }

    #[test]
    fn svg_parsing() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/gradients.svg");
        let res = parse_svg(path);
        assert_eq!(res.len(), 2);

        let d = &res[0];
        assert_eq!(d.id, Some("grad-0".into()));
        assert_eq!(colors2hex(&d.colors), ["#c41189", "#00bfff", "#ffd700"]);
        assert_eq!(d.pos, [0.0, 0.5, 1.0]);

        let d = &res[1];
        assert_eq!(d.id, Some("grad-1".into()));
        assert_eq!(colors2hex(&d.colors), ["#ff1493", "#ffd700", "#2e8b57"]);
        assert_eq!(d.pos, [0.0, 0.5, 1.0]);
    }
}
