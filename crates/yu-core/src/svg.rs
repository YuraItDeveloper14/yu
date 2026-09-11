//! The one renderer: a drawing as an SVG that draws itself.

use crate::draw::{DrawCmd, Drawing, Turtle};

const FONT: &str = "system-ui, -apple-system, Segoe UI, Roboto, sans-serif";

/// A number rounded to two decimals, without trailing zeros; `-0` and non-finite numbers are `0`.
pub fn num(x: f64) -> String {
    let r = (x * 100.0).round() / 100.0;
    if !r.is_finite() || r == 0.0 {
        return "0".into();
    }
    format!("{r}")
}

/// Seconds for CSS, to the millisecond.
fn secs(t: f64) -> String {
    format!("{}s", (t * 1000.0).round() / 1000.0)
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Lines draw themselves in one step; everything else fades in. Before its turn an element is
/// hidden and afterwards it keeps its plain style, so viewers without CSS animation show the
/// finished picture.
fn style(step: f64) -> String {
    let draw = format!(
        ".yu>line{{animation:yu-draw {} linear backwards}}",
        secs(step)
    );
    [
        "<style>",
        ".yu>*{animation:yu-in .3s ease-out backwards}",
        draw.as_str(),
        "@keyframes yu-in{from{opacity:0}}",
        "@keyframes yu-draw{from{stroke-dasharray:1;stroke-dashoffset:1}to{stroke-dasharray:1;stroke-dashoffset:0}}",
        "@media (prefers-reduced-motion:reduce){.yu>*{animation:none}}",
        "</style>",
    ]
    .join("\n")
}

/// A small green turtle where `t` stopped, turned the way it faces.
fn turtle(t: &Turtle, delay: &str) -> String {
    format!(
        "<g transform=\"translate({} {}) rotate({})\"{delay}>\
         <g fill=\"#15803d\"><circle cx=\"6\" cy=\"-8\" r=\"3\"/><circle cx=\"6\" cy=\"8\" r=\"3\"/>\
         <circle cx=\"-6\" cy=\"-8\" r=\"3\"/><circle cx=\"-6\" cy=\"8\" r=\"3\"/>\
         <circle cx=\"12\" cy=\"0\" r=\"4\"/><path d=\"M-9 -3 L-15 0 L-9 3 Z\"/></g>\
         <ellipse rx=\"10\" ry=\"8\" fill=\"#22c55e\" stroke=\"#15803d\" stroke-width=\"1.5\"/>\
         <ellipse rx=\"5\" ry=\"4\" fill=\"#16a34a\"/></g>",
        num(t.x),
        num(t.y),
        num(t.heading)
    )
}

/// The drawing as an animated SVG; the same drawing always gives the same text.
pub fn render(d: &Drawing) -> String {
    let (width, height) = (d.width, d.height);
    let places = d.cmds.len() + usize::from(d.turtle.used);
    let step = (4.0 / places.max(1) as f64).min(0.15);
    let delay = |place: usize| format!(" style=\"animation-delay:{}\"", secs(place as f64 * step));
    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\">\n{}\n\
         <rect width=\"{width}\" height=\"{height}\" fill=\"#ffffff\"/>\n\
         <g class=\"yu\" stroke-linecap=\"round\" stroke-linejoin=\"round\" font-family=\"{FONT}\" font-size=\"20\">\n",
        style(step)
    );
    for (i, cmd) in d.cmds.iter().enumerate() {
        let element = match cmd {
            DrawCmd::Background(color) => format!(
                "<rect width=\"{width}\" height=\"{height}\" fill=\"{}\"{}/>",
                color.hex(),
                delay(i)
            ),
            DrawCmd::Circle { x, y, r, color } => format!(
                "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\"{}/>",
                num(*x),
                num(*y),
                num(*r),
                color.hex(),
                delay(i)
            ),
            DrawCmd::Rect { x, y, w, h, color } => format!(
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\"{}/>",
                num(*x),
                num(*y),
                num(*w),
                num(*h),
                color.hex(),
                delay(i)
            ),
            DrawCmd::Line {
                x1,
                y1,
                x2,
                y2,
                color,
                width,
            } => format!(
                "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"{}\" pathLength=\"1\"{}/>",
                num(*x1),
                num(*y1),
                num(*x2),
                num(*y2),
                color.hex(),
                num(*width),
                delay(i)
            ),
            DrawCmd::Label { text, x, y, color } => format!(
                "<text x=\"{}\" y=\"{}\" fill=\"{}\"{}>{}</text>",
                num(*x),
                num(y + 16.0),
                color.hex(),
                delay(i),
                escape(text)
            ),
            DrawCmd::Fill {
                points,
                color,
                slot,
            } => {
                let points: Vec<String> = points
                    .iter()
                    .map(|(x, y)| format!("{},{}", num(*x), num(*y)))
                    .collect();
                format!(
                    "<polygon points=\"{}\" fill=\"{}\"{}/>",
                    points.join(" "),
                    color.hex(),
                    delay(*slot)
                )
            }
        };
        out.push_str(&element);
        out.push('\n');
    }
    if d.turtle.used {
        out.push_str(&turtle(&d.turtle, &delay(d.cmds.len())));
        out.push('\n');
    }
    out.push_str("</g>\n</svg>\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::Rgb;

    #[test]
    fn numbers_are_short_and_always_valid() {
        assert_eq!(num(300.0), "300");
        assert_eq!(num(212.1320343), "212.13");
        assert_eq!(num(-12.5), "-12.5");
        assert_eq!(num(-0.001), "0");
        assert_eq!(num(f64::INFINITY), "0");
        assert_eq!(num(f64::NAN), "0");
    }

    #[test]
    fn an_empty_drawing_is_a_white_canvas() {
        let svg = render(&Drawing::default());
        assert!(svg.starts_with(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"600\" height=\"400\" viewBox=\"0 0 600 400\">\n<style>\n"
        ));
        assert!(svg.contains("\n<rect width=\"600\" height=\"400\" fill=\"#ffffff\"/>\n"));
        assert!(svg.ends_with("</g>\n</svg>\n"));
        assert!(!svg.contains("rotate("));
    }

    #[test]
    fn shapes_become_elements_with_growing_delays() {
        let mut d = Drawing::default();
        d.circle(300.0, 200.0, 50.0).unwrap();
        d.line(0.0, 0.0, 10.0, 20.0).unwrap();
        d.label("<Привіт & пока>".into(), 5.0, 10.0).unwrap();
        let svg = render(&d);
        assert!(svg.contains(
            "<circle cx=\"300\" cy=\"200\" r=\"50\" fill=\"#000000\" style=\"animation-delay:0s\"/>"
        ));
        assert!(svg.contains(
            "<line x1=\"0\" y1=\"0\" x2=\"10\" y2=\"20\" stroke=\"#000000\" stroke-width=\"2\" pathLength=\"1\" style=\"animation-delay:0.15s\"/>"
        ));
        assert!(svg.contains(
            "<text x=\"5\" y=\"26\" fill=\"#000000\" style=\"animation-delay:0.3s\">&lt;Привіт &amp; пока&gt;</text>"
        ));
    }

    #[test]
    fn backgrounds_and_rectangles() {
        let mut d = Drawing::default();
        d.background(Rgb(0x00, 0x57, 0xb7)).unwrap();
        d.rect(10.0, 20.0, 30.0, 40.0).unwrap();
        let svg = render(&d);
        assert!(svg.contains(
            "<rect width=\"600\" height=\"400\" fill=\"#0057b7\" style=\"animation-delay:0s\"/>"
        ));
        assert!(svg.contains(
            "<rect x=\"10\" y=\"20\" width=\"30\" height=\"40\" fill=\"#000000\" style=\"animation-delay:0.15s\"/>"
        ));
    }

    #[test]
    fn long_drawings_finish_in_about_four_seconds() {
        let mut d = Drawing::default();
        for _ in 0..100 {
            d.line(0.0, 0.0, 1.0, 1.0).unwrap();
        }
        let svg = render(&d);
        assert!(svg.contains(".yu>line{animation:yu-draw 0.04s linear backwards}"));
        assert!(svg.contains("animation-delay:3.96s"));
    }

    #[test]
    fn fills_are_polygons_that_appear_at_their_slot() {
        let mut d = Drawing::default();
        d.begin_fill();
        for _ in 0..3 {
            d.forward(100.0).unwrap();
            d.turn(120.0);
        }
        d.end_fill().unwrap();
        assert!(render(&d).contains(
            "<polygon points=\"300,200 400,200 350,286.6 300,200\" fill=\"#000000\" style=\"animation-delay:0.45s\"/>"
        ));
    }

    #[test]
    fn the_turtle_is_drawn_last_where_it_stopped() {
        let mut d = Drawing::default();
        d.turn(90.0);
        d.forward(50.0).unwrap();
        let svg = render(&d);
        let turtle = svg
            .find("<g transform=\"translate(300 250) rotate(90)\" style=\"animation-delay:0.15s\">")
            .expect("the turtle");
        assert!(svg.find("<line").unwrap() < turtle);
    }
}
