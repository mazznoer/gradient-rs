use colorgrad::{Color, CustomGradient, Gradient};
use svg::node::element::tag::{LinearGradient, RadialGradient, Stop, Type};
use svg::parser::Event;

fn parse_percent_or_float(s: &str) -> Option<f64> {
    if let Some(s) = s.strip_suffix("%") {
        if let Ok(t) = s.parse::<f64>() {
            return Some(t / 100.0);
        }
        return None;
    }

    if let Ok(t) = s.parse::<f64>() {
        return Some(t);
    }

    None
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
    valid: bool,
    id: Option<String>,
    colors: Vec<Color>,
    pos: Vec<f64>,
}

#[allow(non_upper_case_globals)]
pub(crate) fn parse_svg(path: &str) -> Vec<(Gradient, Option<String>)> {
    let mut res = Vec::new();
    let mut index = 0;
    let mut prev_pos = f64::NEG_INFINITY;
    let mut content = String::new();

    for event in svg::open(path, &mut content).unwrap() {
        match event {
            Event::Tag(LinearGradient, t, attributes)
            | Event::Tag(RadialGradient, t, attributes) => match t {
                Type::Start => {
                    let id = if let Some(val) = attributes.get("id") {
                        Some(val.to_string())
                    } else {
                        None
                    };
                    res.push(SvgGradient {
                        valid: true,
                        id,
                        colors: Vec::new(),
                        pos: Vec::new(),
                    });
                }
                Type::End => {
                    index += 1;
                    prev_pos = f64::NEG_INFINITY;
                }
                _ => {}
            },
            Event::Tag(Stop, _, attributes) => {
                if !res[index].valid || res.is_empty() {
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
                    if let Ok(c) = col.parse::<Color>() {
                        Some(c)
                    } else {
                        None
                    }
                } else {
                    None
                };

                let opacity = if let Some(op) = opacity {
                    if let Some(t) = parse_percent_or_float(op) {
                        Some(t)
                    } else {
                        None
                    }
                } else {
                    None
                };

                let offset = if let Some(pos) = attributes.get("offset") {
                    if let Some(pos) = parse_percent_or_float(pos) {
                        Some(pos)
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let (Some(offset), Some(color)) = (offset, color) {
                    let color = if let Some(opacity) = opacity {
                        let (r, g, b, _) = color.rgba();
                        Color::from_rgba(r, g, b, opacity.clamp(0.0, 1.0))
                    } else {
                        color
                    };
                    let position = if offset < prev_pos {
                        prev_pos
                    } else {
                        prev_pos = offset;
                        offset
                    };
                    res[index].colors.push(color);
                    res[index].pos.push(position);
                } else {
                    // Invalid stop, mark the gradient as invalid
                    res[index].valid = false;
                }
            }
            _ => {}
        }
    }

    let mut gradients = Vec::new();

    for g in res {
        if !g.valid {
            continue;
        }

        let grad = CustomGradient::new()
            .colors(&g.colors)
            .domain(&g.pos)
            .build();

        if let Ok(grad) = grad {
            gradients.push((grad, g.id));
        }
    }

    gradients
}
