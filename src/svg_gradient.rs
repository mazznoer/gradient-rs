use colorgrad::Color;
use colorgrad::GradientBuilder;
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
pub struct SvgGradient {
    pub id: Option<String>,
    pub colors: Vec<Color>,
    pub pos: Vec<f32>,
    pub valid: bool,
}

impl SvgGradient {
    pub fn gradient_builder(&mut self) -> Option<GradientBuilder> {
        if !self.valid || self.colors.is_empty() {
            return None;
        }
        if self.pos[0] > 0.0 {
            self.pos.insert(0, 0.0);
            self.colors.insert(0, self.colors[0].clone());
        }
        let last = self.colors.len() - 1;
        if self.pos[last] < 1.0 {
            self.pos.push(1.0);
            self.colors.push(self.colors[last].clone());
        }
        let mut gb = GradientBuilder::new();
        gb.colors(&self.colors);
        gb.domain(&self.pos);
        Some(gb)
    }
}

pub fn parse_svg(s: &str, target_id: Option<&str>) -> Vec<SvgGradient> {
    let mut res = Vec::new();
    let mut index = 0;
    let mut prev_pos = f32::NEG_INFINITY;
    let mut inside = false;
    let mut skip = false;

    for event in svg::read(s).unwrap() {
        match event {
            Event::Tag(svg_tag::LinearGradient, t, attributes)
            | Event::Tag(svg_tag::RadialGradient, t, attributes) => match t {
                svg_tag::Type::Start => {
                    let id = attributes.get("id").map(|v| v.to_string());
                    skip = match (id.as_ref(), target_id) {
                        (Some(a), Some(b)) => a != b,
                        (None, Some(_)) => true,
                        _ => false,
                    };
                    if skip {
                        continue;
                    }
                    inside = true;
                    res.push(SvgGradient {
                        id,
                        colors: Vec::new(),
                        pos: Vec::new(),
                        valid: true,
                    });
                }

                svg_tag::Type::End => {
                    if inside && !skip {
                        index += 1;
                    }
                    inside = false;
                    prev_pos = f32::NEG_INFINITY;
                }

                svg_tag::Type::Empty => {}
            },
            Event::Tag(svg_tag::Stop, _, attributes) => {
                if !inside || skip || res.is_empty() {
                    continue;
                }

                let mut color: Option<Color> = None;
                let mut opacity: Option<f32> = None;

                if let Some(s) = attributes.get("stop-color") {
                    let Ok(c) = s.parse::<Color>() else {
                        res[index].valid = false;
                        continue;
                    };
                    color = Some(c);
                }

                if let Some(s) = attributes.get("stop-opacity") {
                    let Some(opc) = parse_percent_or_float(s) else {
                        res[index].valid = false;
                        continue;
                    };
                    opacity = Some(opc);
                }

                if let Some(styles) = attributes.get("style") {
                    let (col, opac) = parse_styles(styles);

                    if let Some(s) = col {
                        let Ok(c) = s.parse::<Color>() else {
                            res[index].valid = false;
                            continue;
                        };
                        color = Some(c);
                    }

                    if let Some(s) = opac {
                        let Some(opc) = parse_percent_or_float(s) else {
                            res[index].valid = false;
                            continue;
                        };
                        opacity = Some(opc);
                    }
                }

                let offset = if let Some(pos) = attributes.get("offset") {
                    let Some(of) = parse_percent_or_float(pos) else {
                        res[index].valid = false;
                        continue;
                    };
                    Some(of)
                } else {
                    None
                };

                let color = color.unwrap_or(Color::new(0.0, 0.0, 0.0, 1.0));

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

#[cfg(test)]
mod tests {
    use super::*;

    fn colors2hex(colors: &[Color]) -> Vec<String> {
        colors.iter().map(|c| c.to_hex_string()).collect()
    }

    fn str_colors2hex(colors: &[&str]) -> Vec<String> {
        colors
            .iter()
            .map(|s| s.parse::<Color>().unwrap().to_hex_string())
            .collect()
    }

    macro_rules! assert_gradient {
        ($sg:expr, $id:expr, $colors:expr, $pos:expr) => {
            assert_eq!($sg.id, Some($id.into()));
            assert_eq!(colors2hex(&$sg.colors), str_colors2hex($colors));
            assert_eq!(&$sg.pos, $pos);
            assert!($sg.valid);
        };
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
        let result = parse_svg(
            r##"
        <linearGradient id="banana">
            <stop offset="0" stop-color="#C41189" />
            <stop offset="0.5" stop-color="#00BFFF" />
            <stop offset="1" stop-color="#FFD700" />
        </linearGradient>
        "##,
            None,
        );
        assert_eq!(result.len(), 1);
        assert_gradient!(
            result[0],
            "banana",
            &["#c41189", "#00bfff", "#ffd700"],
            &[0.0, 0.5, 1.0]
        );

        // Using percentage
        let result = parse_svg(
            r##"
        <linearGradient id="apple">
            <stop offset="0%" stop-color="deeppink" />
            <stop offset="50%" stop-color="gold" />
            <stop offset="100%" stop-color="seagreen" />
        </linearGradient>
        "##,
            None,
        );
        assert_eq!(result.len(), 1);
        assert_gradient!(
            result[0],
            "apple",
            &["deeppink", "gold", "seagreen"],
            &[0.0, 0.5, 1.0]
        );

        // radialGradient tag
        let result = parse_svg(
            r##"
        <radialGradient id="mango">
            <stop offset="0" stop-color="deeppink" />
            <stop offset="0.5" stop-color="gold" />
            <stop offset="1" stop-color="seagreen" />
        </radialGradient>
        "##,
            None,
        );
        assert_eq!(result.len(), 1);
        assert_gradient!(
            result[0],
            "mango",
            &["deeppink", "gold", "seagreen"],
            &[0.0, 0.5, 1.0]
        );

        fn set_alpha(col: &str, alpha: f32) -> String {
            let c = col.parse::<Color>().unwrap();
            Color::new(c.r, c.g, c.b, alpha).to_hex_string()
        }

        // Using style attribute
        let result = parse_svg(
            r##"
        <linearGradient id="papaya">
            <stop offset="0" style="stop-color:tomato;" />
            <stop offset="0.5" style="stop-color:gold;stop-opacity:0.9;" />
            <stop offset="1" style="stop-color:steelblue;" />
        </linearGradient>
        "##,
            None,
        );
        assert_eq!(result.len(), 1);
        assert_gradient!(
            result[0],
            "papaya",
            &["tomato", &set_alpha("gold", 0.9), "steelblue"],
            &[0.0, 0.5, 1.0]
        );

        // Multiple gradients
        let result = parse_svg(
            r##"
        <linearGradient id="gradient-1">
            <stop offset="0" stop-color="#c4114d" />
            <stop offset="0.5" stop-color="#6268a6" />
            <stop offset="0.5" stop-color="#57cf4f" />
            <stop offset="1" stop-color="#ffe04d" />
        </linearGradient>
        <!-- This should render just like #gradient-1 -->
        <linearGradient id="gradient-2">
            <stop offset="0" stop-color="#c4114d" />
            <stop offset="0.5" stop-color="#6268a6" />
            <stop offset="0.2" stop-color="#57cf4f" />
            <stop offset="1" stop-color="#ffe04d" />
        </linearGradient>
        "##,
            None,
        );
        assert_eq!(result.len(), 2);
        assert_gradient!(
            result[0],
            "gradient-1",
            &["#c4114d", "#6268a6", "#57cf4f", "#ffe04d"],
            &[0.0, 0.5, 0.5, 1.0]
        );
        assert_gradient!(
            result[1],
            "gradient-2",
            &["#c4114d", "#6268a6", "#57cf4f", "#ffe04d"],
            &[0.0, 0.5, 0.5, 1.0]
        );

        // Incomplete stop attributes
        let result = parse_svg(
            r##"
        <linearGradient id="g4657">
            <stop offset="0" />
            <stop offset="0.7" stop-color="gold" />
            <stop stop-color="steelblue" />
        </linearGradient>
        "##,
            None,
        );
        assert_eq!(result.len(), 1);
        assert_gradient!(
            result[0],
            "g4657",
            &["black", "gold", "steelblue"],
            &[0.0, 0.7, 0.7]
        );
    }

    #[test]
    fn filter_by_id() {
        let s = r##"
        <linearGradient id="guava">
            <stop offset="0.1" stop-color="#f00" />
            <stop offset="0.5" stop-color="#0f0" />
            <stop offset="0.9" stop-color="#00f" />
        </linearGradient>
        
        <linearGradient id="avocado">
            <stop offset="20%" stop-color="#ff0" />
            <stop offset="40%" stop-color="#f0f" />
            <stop offset="80%" stop-color="#0ff" />
        </linearGradient>
        
        <linearGradient>
            <stop offset="0" stop-color="#0f0" />
        </linearGradient>
        
        <linearGradient id="">
            <stop offset="1" stop-color="#123456" />
        </linearGradient>
        
        <radialGradient id="guava">
            <stop offset="35%" stop-color="#abc123" />
        </radialGradient>
        "##;

        let result = parse_svg(s, None);
        assert_eq!(result.len(), 5);

        let result = parse_svg(s, Some("guava"));
        assert_eq!(result.len(), 2);
        assert_gradient!(
            result[0],
            "guava",
            &["#ff0000", "#00ff00", "#0000ff"],
            &[0.1, 0.5, 0.9]
        );
        assert_gradient!(result[1], "guava", &["#abc123"], &[0.35]);

        let result = parse_svg(s, Some("avocado"));
        assert_eq!(result.len(), 1);
        assert_gradient!(
            result[0],
            "avocado",
            &["#ffff00", "#ff00ff", "#00ffff"],
            &[0.2, 0.4, 0.8]
        );

        let result = parse_svg(s, Some(""));
        assert_eq!(result.len(), 1);
        assert_gradient!(result[0], "", &["#123456"], &[1.0]);

        let result = parse_svg(s, Some("pineapple"));
        assert!(result.is_empty());
    }

    #[test]
    fn malformed_gradients() {
        let result = parse_svg(
            r##"
        <!-- orphaned stop will be ignored -->
        <stop offset="0" stop-color="pink" />
        
        <linearGradient id="empty">
        </linearGradient>
        
        <stop offset="0" stop-color="red" />
        
        <linearGradient id="empty-stops">
            <stop />
            <stop />
            <stop />
        </linearGradient>
        
        <stop offset="0" stop-color="gold" />
        <stop offset="0" stop-color="plum" />
        
        <linearGradient id="without-color">
            <stop offset="0%" />
            <stop offset="75%" />
            <stop offset="100%" />
        </linearGradient>
        
        <linearGradient id="without-offset">
            <stop stop-color="red" />
            <stop stop-color="lime" />
            <stop stop-color="blue" />
        </linearGradient>
        "##,
            None,
        );
        assert_eq!(result.len(), 4);
        assert_gradient!(result[0], "empty", &[], &[]);
        assert_gradient!(
            result[1],
            "empty-stops",
            &["black", "black", "black"],
            &[0.0, 0.0, 0.0]
        );
        assert_gradient!(
            result[2],
            "without-color",
            &["black", "black", "black"],
            &[0.0, 0.75, 1.0]
        );
        assert_gradient!(
            result[3],
            "without-offset",
            &["red", "lime", "blue"],
            &[0.0, 0.0, 0.0]
        );
    }

    #[test]
    fn invalid_gradients() {
        let result = parse_svg(
            r##"
        <!-- invalid color -->
        
        <linearGradient>
            <stop offset="50%" stop-color="stone" />
        </linearGradient>
        
        <linearGradient>
            <stop offset="50%" style="stop-color:#zzz;" />
        </linearGradient>
        
        <!-- invalid offset -->
        
        <linearGradient>
            <stop offset="5x%" stop-color="gold" />
        </linearGradient>
        
        <!-- invalid color & offset -->
        
        <linearGradient>
            <stop offset="x" stop-color="stone" />
        </linearGradient>
        
        <!-- invalid opacity -->
        
        <linearGradient>
            <stop offset="50%" stop-color="red" stop-opacity="0.5x" />
        </linearGradient>
        
        <linearGradient>
            <stop offset="50%" stop-color="red" style="stop-opacity:%;" />
        </linearGradient>
        "##,
            None,
        );
        assert_eq!(result.len(), 6);
        for g in &result {
            assert_eq!(g.valid, false);
            assert_eq!(g.id, None);
        }
    }
}
