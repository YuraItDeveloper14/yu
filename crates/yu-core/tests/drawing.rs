mod common;

use common::Capture;
use yu_core::diagnostic::Ty;
use yu_core::{DrawCmd, Drawing, ErrorKind, Lang, Options, Rgb, Session};

const RED: Rgb = Rgb(0xef, 0x44, 0x44);

/// Runs `src` in a new session and returns what it drew.
fn draw(src: &str) -> Drawing {
    let mut session = Session::new(Options::default());
    if let Err(e) = session.run(src, &mut Capture::default()) {
        panic!("{}", e.render(src, Lang::Uk));
    }
    session.drawing().clone()
}

fn err(src: &str) -> ErrorKind {
    let mut session = Session::new(Options::default());
    session.run(src, &mut Capture::default()).unwrap_err().kind
}

#[test]
fn shapes_in_both_languages() {
    let d = draw("колір(\"червона\")\nколо(300, 200, 50)\ncolor(\"#0057B7\")\nrect(0, 0, 10, 20)");
    assert_eq!(
        d.cmds,
        vec![
            DrawCmd::Circle {
                x: 300.0,
                y: 200.0,
                r: 50.0,
                color: RED
            },
            DrawCmd::Rect {
                x: 0.0,
                y: 0.0,
                w: 10.0,
                h: 20.0,
                color: Rgb(0x00, 0x57, 0xb7)
            },
        ]
    );
}

#[test]
fn labels_show_values_like_say() {
    let d = draw("напис(\"Бал: \" + 5, 10, 20)\nlabel(так, 0, 0)");
    assert!(matches!(&d.cmds[0], DrawCmd::Label { text, .. } if text == "Бал: 5"));
    assert!(matches!(&d.cmds[1], DrawCmd::Label { text, .. } if text == "так"));
}

#[test]
fn the_turtle_draws_a_square() {
    let d = draw("повтори 4 рази:\n    вперед(100)\n    праворуч(90)");
    assert_eq!(d.cmds.len(), 4);
    assert!(d.turtle.used);
    assert!(matches!(d.cmds[1], DrawCmd::Line { x1, y1, .. } if x1 == 400.0 && y1 == 200.0));
}

#[test]
fn a_filled_triangle_in_english() {
    let d = draw(
        "color(\"yellow\")\nbegin_fill()\nrepeat 3 times:\n    forward(100)\n    left(120)\nend_fill()\npen_up()\nback(10)",
    );
    assert!(matches!(&d.cmds[0], DrawCmd::Fill { color, points, .. }
        if *color == Rgb(0xff, 0xd7, 0x00) && points.len() == 4));
    assert_eq!(d.cmds.len(), 4);
}

#[test]
fn a_random_colour_never_repeats_the_current_one() {
    let d = draw(
        "колір(\"червоний\")\nповтори 20 разів:\n    колір(\"випадковий\")\n    коло(0, 0, 1)",
    );
    let colors: Vec<Rgb> = d
        .cmds
        .iter()
        .map(|c| match c {
            DrawCmd::Circle { color, .. } => *color,
            other => panic!("{other:?}"),
        })
        .collect();
    assert_ne!(colors[0], RED);
    assert!(colors.windows(2).all(|w| w[0] != w[1]));
}

#[test]
fn drawing_errors() {
    assert_eq!(
        err("колір(червоний)"),
        ErrorKind::ColorNeedsQuotes("червоний".into())
    );
    assert_eq!(err("фон(5)"), ErrorKind::ExpectedText(Ty::Number));
    assert_eq!(
        err("колір(\"червний\")"),
        ErrorKind::UnknownColor {
            name: "червний".into(),
            suggestion: Some("червоний".into())
        }
    );
    assert_eq!(
        err("коло(1, 2)"),
        ErrorKind::ArgCount {
            name: "коло".into(),
            expected: 3,
            got: 2
        }
    );
    assert_eq!(err("коло(1, 2, -3)"), ErrorKind::NegativeSize(-3.0));
    assert_eq!(
        err("коло(\"1\", 2, 3)"),
        ErrorKind::ExpectedNumber(Ty::Text)
    );
    assert_eq!(err("заверши_заливку()"), ErrorKind::FillNotStarted);
    assert_eq!(err("полотно(0, 10)"), ErrorKind::BadCanvas);
    assert_eq!(err("лінія = 5"), ErrorKind::BuiltinAsName("лінія".into()));
}

#[test]
fn a_session_keeps_its_drawing_and_its_random_sequence() {
    let mut session = Session::new(Options::default());
    let mut host = Capture::default();
    session.run("коло(10, 10, 5)", &mut host).unwrap();
    session.run("коло(20, 20, 5)", &mut host).unwrap();
    assert_eq!(session.drawing().cmds.len(), 2);
    let first = session.run("випадкове(1, 1000000)", &mut host).unwrap();
    let second = session.run("випадкове(1, 1000000)", &mut host).unwrap();
    assert_ne!(first, second);
}
