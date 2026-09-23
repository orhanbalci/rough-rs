use std::borrow::Borrow;

use svgtypes::PathSegment;

/// Options for [`write_path`].
///
/// The default writes every number as it is, with a space after each command
/// and between numbers: `M 10 10 L 20.5 20`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WriteOptions {
    /// Round numbers to this many decimal places. `None` writes them unrounded.
    pub precision: Option<u8>,
    /// Write the shortest equivalent path data: no space after a command,
    /// separators between numbers only where needed, no leading zeros, and no
    /// command letter when the previous segment used the same one, as in
    /// `M10 10l.5-5 2 0`. Move commands always keep their letter, since a
    /// repeated one would otherwise read as a line.
    pub compact: bool,
}

/// Writes path segments as SVG path data.
///
/// ```
/// use svg_path_ops::{write_path, PathSegment, WriteOptions};
///
/// let segments = [
///     PathSegment::MoveTo { abs: true, x: 10.0, y: 10.0 },
///     PathSegment::LineTo { abs: false, x: 0.5, y: -5.0 },
///     PathSegment::LineTo { abs: false, x: 2.0, y: 0.0 },
/// ];
///
/// let default = write_path(&segments, &WriteOptions::default());
/// assert_eq!(default, "M 10 10 l 0.5 -5 l 2 0");
///
/// let compact = WriteOptions { compact: true, ..WriteOptions::default() };
/// assert_eq!(write_path(&segments, &compact), "M10 10l.5-5 2 0");
/// ```
pub fn write_path(
    segments: impl IntoIterator<Item = impl Borrow<PathSegment>>,
    options: &WriteOptions,
) -> String {
    let mut writer = PathWriter {
        options,
        out: String::new(),
        prev_command: None,
        prev_number_has_dot: None,
    };
    for segment in segments {
        writer.segment(segment.borrow());
    }
    writer.out
}

struct PathWriter<'a> {
    options: &'a WriteOptions,
    out: String,
    prev_command: Option<u8>,
    /// Whether the last token written was a number, and if so whether it
    /// contained a decimal point.
    prev_number_has_dot: Option<bool>,
}

impl PathWriter<'_> {
    fn segment(&mut self, segment: &PathSegment) {
        let command = segment.command();
        let repeats =
            self.prev_command == Some(command) && !matches!(command, b'M' | b'm' | b'Z' | b'z');

        if !self.options.compact || !repeats {
            if !self.options.compact && !self.out.is_empty() {
                self.out.push(' ');
            }
            self.out.push(command as char);
            self.prev_number_has_dot = None;
        }
        self.prev_command = Some(command);

        match *segment {
            PathSegment::MoveTo { x, y, .. }
            | PathSegment::LineTo { x, y, .. }
            | PathSegment::SmoothQuadratic { x, y, .. } => self.numbers(&[x, y]),
            PathSegment::HorizontalLineTo { x, .. } => self.numbers(&[x]),
            PathSegment::VerticalLineTo { y, .. } => self.numbers(&[y]),
            PathSegment::CurveTo { x1, y1, x2, y2, x, y, .. } => {
                self.numbers(&[x1, y1, x2, y2, x, y])
            }
            PathSegment::SmoothCurveTo { x2, y2, x, y, .. } => self.numbers(&[x2, y2, x, y]),
            PathSegment::Quadratic { x1, y1, x, y, .. } => self.numbers(&[x1, y1, x, y]),
            PathSegment::EllipticalArc {
                rx, ry, x_axis_rotation, large_arc, sweep, x, y, ..
            } => self.numbers(&[
                rx,
                ry,
                x_axis_rotation,
                f64::from(u8::from(large_arc)),
                f64::from(u8::from(sweep)),
                x,
                y,
            ]),
            PathSegment::ClosePath { .. } => {}
        }
    }

    fn numbers(&mut self, values: &[f64]) {
        for &value in values {
            self.number(value);
        }
    }

    fn number(&mut self, value: f64) {
        let value = match self.options.precision {
            Some(precision) => round(value, precision),
            None => value,
        };
        let mut text = value.to_string();

        if self.options.compact {
            if let Some(fraction) = text.strip_prefix("0.") {
                text = format!(".{fraction}");
            } else if let Some(fraction) = text.strip_prefix("-0.") {
                text = format!("-.{fraction}");
            }
            // A sign, or a second decimal point, already ends the previous number.
            let needs_separator = match self.prev_number_has_dot {
                None => false,
                Some(prev_has_dot) => {
                    !(text.starts_with('-') || (prev_has_dot && text.starts_with('.')))
                }
            };
            if needs_separator {
                self.out.push(' ');
            }
        } else {
            self.out.push(' ');
        }

        self.prev_number_has_dot = Some(text.contains('.'));
        self.out.push_str(&text);
    }
}

fn round(value: f64, precision: u8) -> f64 {
    let factor = 10.0f64.powi(i32::from(precision));
    let rounded = (value * factor).round() / factor;
    if !rounded.is_finite() {
        // The factor overflowed; the value already has fewer decimal places.
        value
    } else if rounded == 0.0 {
        // Avoid writing "-0".
        0.0
    } else {
        rounded
    }
}

#[cfg(test)]
mod test {
    use svgtypes::PathParser;

    use super::{write_path, WriteOptions};
    use crate::PathSegment;

    fn parse(path: &str) -> Vec<PathSegment> {
        PathParser::from(path).map(Result::unwrap).collect()
    }

    fn compact() -> WriteOptions {
        WriteOptions { compact: true, ..WriteOptions::default() }
    }

    #[test]
    fn default_writes_spaced_commands() {
        let segments = parse("M10 10 H20 v-5 C1 2 3 4 5 6 a5 5 30 1 0 10 10 z");
        assert_eq!(
            write_path(&segments, &WriteOptions::default()),
            "M 10 10 H 20 v -5 C 1 2 3 4 5 6 a 5 5 30 1 0 10 10 z"
        );
    }

    #[test]
    fn empty_path_writes_empty_string() {
        let segments: Vec<PathSegment> = Vec::new();
        assert_eq!(write_path(&segments, &WriteOptions::default()), "");
        assert_eq!(write_path(&segments, &compact()), "");
    }

    #[test]
    fn precision_rounds_numbers() {
        let segments = parse("M1.23456 -0.0001 L2.5 3");
        let options = WriteOptions { precision: Some(2), ..WriteOptions::default() };
        assert_eq!(write_path(&segments, &options), "M 1.23 0 L 2.5 3");
    }

    #[test]
    fn precision_near_u8_max_keeps_value() {
        let segments = parse("M1.5 2");
        let options = WriteOptions {
            precision: Some(u8::MAX),
            ..WriteOptions::default()
        };
        assert_eq!(write_path(&segments, &options), "M 1.5 2");
    }

    #[test]
    fn compact_drops_redundant_separators() {
        let segments = parse("M 10 10 L 0.5 -0.5 L 0.25 0.75 Z");
        assert_eq!(write_path(&segments, &compact()), "M10 10L.5-.5.25.75Z");
    }

    #[test]
    fn compact_keeps_repeated_move_commands() {
        let segments = parse("M1 1 M2 2 m3 3 m4 4");
        assert_eq!(write_path(&segments, &compact()), "M1 1M2 2m3 3m4 4");
    }

    #[test]
    fn compact_keeps_letter_when_case_changes() {
        let segments = parse("M0 0 L1 1 l2 2 l3 3");
        assert_eq!(write_path(&segments, &compact()), "M0 0L1 1l2 2 3 3");
    }

    #[test]
    fn compact_output_parses_back_to_same_segments() {
        let segments =
            parse("M10 10 L0.5 -0.5 l.25 .75 C1 2 3 4 5 6 S-1-2-3-4 a5 5 30 1 0 10 10 h-3 z");
        let written = write_path(&segments, &compact());
        assert_eq!(parse(&written), segments);
    }
}
