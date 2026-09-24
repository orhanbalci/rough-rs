use std::borrow::Borrow;

use kurbo::{CubicBez, ParamCurve, ParamCurveArclen, ParamCurveArea, PathEl, Point};
use svgtypes::PathSegment;

use crate::measure::{PathMeasure, Piece, PieceShape};

/// Accuracy of arc lengths when splitting curves in half.
const ARCLEN_ACCURACY: f64 = 1e-9;

/// Two paths brought to the same shape of path data, so every shape
/// between them is a matter of mixing their numbers: a morph from one to
/// the other.
///
/// ```
/// use svg_path_ops::svgtypes::PathParser;
/// use svg_path_ops::{Morph, PathMeasure};
///
/// let parse = |data| PathParser::from(data).collect::<Result<Vec<_>, _>>();
/// let square = parse("M 0 0 H 20 V 20 H 0 Z")?;
/// let circle = parse("M 20 10 A 10 10 0 0 1 0 10 A 10 10 0 0 1 20 10 Z")?;
///
/// let morph = Morph::new(&square, &circle);
/// // The ends are the paths themselves
/// assert!((PathMeasure::new(morph.at(0.0)).area() - 400.0).abs() < 1e-6);
/// let circle_area = std::f64::consts::PI * 100.0;
/// assert!((PathMeasure::new(morph.at(1.0)).area() - circle_area).abs() < 1e-3);
/// // Halfway, a rounded square
/// let halfway = PathMeasure::new(morph.at(0.5)).area();
/// assert!(circle_area < halfway && halfway < 400.0);
/// # Ok::<(), svg_path_ops::svgtypes::Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct Morph {
    from: Vec<Subpath>,
    to: Vec<Subpath>,
}

/// A subpath as cubic curves.
#[derive(Clone, Debug)]
struct Subpath {
    start: Point,
    curves: Vec<CubicBez>,
    closed: bool,
}

impl Morph {
    /// Brings `from` and `to` to the same shape of path data.
    ///
    /// # Algorithm
    ///
    /// Every segment becomes a cubic curve: lines and quadratic curves
    /// exactly, arcs within a millionth of their radius. Subpaths are paired
    /// in order; one without a partner grows from, or shrinks to, a point in
    /// the middle of its partner-to-be's bounds. A subpath with fewer curves
    /// than its partner has its longest curve cut in half by length, again
    /// and again, which leaves its shape as it was. Where both are closed,
    /// one is turned to run the same way as the other if it does not, and
    /// its start is moved round to the curve that puts the two subpaths'
    /// points closest together, by the sum of the squared distances between
    /// them, so the shape does not twist on its way.
    ///
    /// So a subpath of the second path may come out running the other way
    /// than it did, which changes what it fills only where holes depend on
    /// the direction they run, under the nonzero rule. Turning both paths
    /// with [`reorient`](crate::reorient) first keeps outlines and holes
    /// running their own ways.
    pub fn new(
        from: impl IntoIterator<Item = impl Borrow<PathSegment>>,
        to: impl IntoIterator<Item = impl Borrow<PathSegment>>,
    ) -> Self {
        let mut from = subpaths(&PathMeasure::new(from));
        let mut to = subpaths(&PathMeasure::new(to));

        // A subpath without a partner comes from a point
        while from.len() < to.len() {
            let partner = &to[from.len()];
            from.push(point_like(partner));
        }
        while to.len() < from.len() {
            let partner = &from[to.len()];
            to.push(point_like(partner));
        }

        for (a, b) in from.iter_mut().zip(to.iter_mut()) {
            let count = a.curves.len().max(b.curves.len()).max(1);
            split_to(a, count);
            split_to(b, count);
            if a.closed && b.closed {
                align(a, b);
            }
            // Both close only when both did
            let closed = a.closed && b.closed;
            a.closed = closed;
            b.closed = closed;
        }
        Morph { from, to }
    }

    /// The path `t` of the way from the first path to the second: the first
    /// at 0, the second at 1, and beyond them as the numbers carry on. It is
    /// in absolute coordinates, made of moves, cubic curves and close paths.
    /// A subpath that grows from a point, or shrinks to one, is still there
    /// at that end, as a subpath that draws nothing.
    pub fn at(&self, t: f64) -> Vec<PathSegment> {
        let mix = |a: Point, b: Point| a.lerp(b, t);
        let mut path = Vec::new();
        for (a, b) in self.from.iter().zip(&self.to) {
            let start = mix(a.start, b.start);
            path.push(PathSegment::MoveTo { abs: true, x: start.x, y: start.y });
            for (c, d) in a.curves.iter().zip(&b.curves) {
                let (p1, p2, p3) = (mix(c.p1, d.p1), mix(c.p2, d.p2), mix(c.p3, d.p3));
                path.push(PathSegment::CurveTo {
                    abs: true,
                    x1: p1.x,
                    y1: p1.y,
                    x2: p2.x,
                    y2: p2.y,
                    x: p3.x,
                    y: p3.y,
                });
            }
            if a.closed {
                path.push(PathSegment::ClosePath { abs: true });
            }
        }
        path
    }
}

/// The path `t` of the way from `from` to `to`, as Paper.js's
/// `interpolate` gives it. See [`Morph`], which keeps the work of matching
/// the paths up for when many steps are wanted, as in an animation.
///
/// ```
/// use svg_path_ops::svgtypes::PathParser;
/// use svg_path_ops::{interpolate, write_path, WriteOptions};
///
/// let parse = |data| PathParser::from(data).collect::<Result<Vec<_>, _>>();
/// let low = parse("M 0 0 L 10 0")?;
/// let high = parse("M 0 10 L 10 10")?;
/// let middle = interpolate(&low, &high, 0.5);
/// assert_eq!(
///     write_path(
///         &middle,
///         &WriteOptions { precision: Some(6), ..WriteOptions::default() }
///     ),
///     "M 0 5 C 3.333333 5 6.666667 5 10 5"
/// );
/// # Ok::<(), svg_path_ops::svgtypes::Error>(())
/// ```
pub fn interpolate(
    from: impl IntoIterator<Item = impl Borrow<PathSegment>>,
    to: impl IntoIterator<Item = impl Borrow<PathSegment>>,
    t: f64,
) -> Vec<PathSegment> {
    Morph::new(from, to).at(t)
}

/// The subpaths of the path measured by `measure`, as cubic curves.
fn subpaths(measure: &PathMeasure) -> Vec<Subpath> {
    measure
        .subpaths()
        .map(|pieces| Subpath {
            start: pieces[0].from,
            curves: pieces
                .iter()
                .filter(|p| p.length > 0.0)
                .flat_map(cubics)
                .collect(),
            closed: pieces.last().is_some_and(|piece| piece.closes),
        })
        .collect()
}

/// The cubic curves that draw `piece`.
fn cubics(piece: &Piece) -> Vec<CubicBez> {
    match piece.shape {
        PieceShape::Line(line) => {
            vec![CubicBez::new(
                line.p0,
                line.p0.lerp(line.p1, 1.0 / 3.0),
                line.p0.lerp(line.p1, 2.0 / 3.0),
                line.p1,
            )]
        }
        PieceShape::Quadratic(quad) => vec![quad.raise()],
        PieceShape::Cubic(cubic) => vec![cubic],
        PieceShape::Arc(arc) => {
            let accuracy = 1e-6 * arc.radii.x.max(arc.radii.y);
            let mut at = arc.eval(0.0);
            let mut curves = Vec::new();
            for element in arc.append_iter(accuracy) {
                if let PathEl::CurveTo(p1, p2, p3) = element {
                    curves.push(CubicBez::new(at, p1, p2, p3));
                    at = p3;
                }
            }
            // End exactly where the path says
            if let Some(last) = curves.last_mut() {
                last.p3 = piece.to;
            }
            if let Some(first) = curves.first_mut() {
                first.p0 = piece.from;
            }
            curves
        }
    }
}

/// A subpath of as many curves as `partner`, all at the middle of its
/// bounds, which grows into it or shrinks from it.
fn point_like(partner: &Subpath) -> Subpath {
    let points = std::iter::once(partner.start)
        .chain(partner.curves.iter().flat_map(|c| [c.p1, c.p2, c.p3]));
    let (low, high) = points.fold(
        (
            Point::new(f64::INFINITY, f64::INFINITY),
            Point::new(f64::NEG_INFINITY, f64::NEG_INFINITY),
        ),
        |(low, high), p| {
            (
                Point::new(low.x.min(p.x), low.y.min(p.y)),
                Point::new(high.x.max(p.x), high.y.max(p.y)),
            )
        },
    );
    let middle = low.midpoint(high);
    Subpath {
        start: middle,
        curves: vec![CubicBez::new(middle, middle, middle, middle); partner.curves.len()],
        closed: partner.closed,
    }
}

/// Cuts the longest curves of `subpath` in half until it has `count`.
fn split_to(subpath: &mut Subpath, count: usize) {
    if subpath.curves.is_empty() {
        // A subpath that draws nothing is a point
        let at = subpath.start;
        subpath.curves = vec![CubicBez::new(at, at, at, at); count];
        return;
    }
    let mut lengths: Vec<f64> = subpath
        .curves
        .iter()
        .map(|c| c.arclen(ARCLEN_ACCURACY))
        .collect();
    while subpath.curves.len() < count {
        let (longest, &length) = lengths
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .expect("a subpath has curves");
        let curve = subpath.curves[longest];
        let t = curve.inv_arclen(length / 2.0, ARCLEN_ACCURACY);
        let (first, second) = (curve.subsegment(0.0..t), curve.subsegment(t..1.0));
        subpath.curves.splice(longest..=longest, [first, second]);
        lengths.splice(longest..=longest, [length / 2.0, length / 2.0]);
    }
}

/// Turns and rotates the closed subpath `b` so its points line up with
/// those of `a`, which has as many curves.
fn align(a: &Subpath, b: &mut Subpath) {
    let n = b.curves.len();
    if n == 0 {
        return;
    }
    // Run the same way; a point runs no way at all
    let (area_a, area_b) = (signed_area(a), signed_area(b));
    if area_a != 0.0 && area_b != 0.0 && (area_a > 0.0) != (area_b > 0.0) {
        b.curves = b
            .curves
            .iter()
            .rev()
            .map(|c| CubicBez::new(c.p3, c.p2, c.p1, c.p0))
            .collect();
    }
    // Start at the curve whose points lie closest to `a`'s
    let ends_a: Vec<Point> = a.curves.iter().map(|c| c.p0).collect();
    let ends_b: Vec<Point> = b.curves.iter().map(|c| c.p0).collect();
    let cost = |shift: usize| -> f64 {
        (0..n)
            .map(|i| (ends_a[i] - ends_b[(i + shift) % n]).hypot2())
            .sum()
    };
    let shift = (0..n)
        .min_by(|x, y| cost(*x).total_cmp(&cost(*y)))
        .unwrap_or(0);
    b.curves.rotate_left(shift);
    b.start = b.curves[0].p0;
}

/// The area a closed subpath encloses, signed by its direction.
fn signed_area(subpath: &Subpath) -> f64 {
    subpath.curves.iter().map(|c| c.signed_area()).sum()
}

#[cfg(test)]
mod test {
    use svgtypes::{PathParser, PathSegment};

    use super::{interpolate, Morph};
    use crate::{split_subpaths, PathMeasure};

    fn parse(path: &str) -> Vec<PathSegment> {
        PathParser::from(path).collect::<Result<_, _>>().unwrap()
    }

    /// The furthest any point of `a` is from `b`, both ways.
    fn distance(a: &[PathSegment], b: &[PathSegment]) -> f64 {
        let one_way = |a: &[PathSegment], b: &[PathSegment]| {
            let parts: Vec<PathMeasure> = split_subpaths(b).iter().map(PathMeasure::new).collect();
            let mut worst = 0.0f64;
            for subpath in split_subpaths(a) {
                let measure = PathMeasure::new(&subpath);
                let n = (measure.total_length() / 0.1).ceil().max(1.0) as usize;
                for k in 0..=n {
                    if let Some(p) = measure.point_at(measure.total_length() * k as f64 / n as f64)
                    {
                        let d = parts
                            .iter()
                            .filter_map(|m| m.nearest(p))
                            .map(|n| n.distance)
                            .fold(f64::INFINITY, f64::min);
                        worst = worst.max(d);
                    }
                }
            }
            worst
        };
        one_way(a, b).max(one_way(b, a))
    }

    const SHAPES: [&str; 6] = [
        "M 0 0 H 40 V 30 H 0 Z",
        "M 40 20 A 20 20 0 0 1 0 20 A 20 20 0 0 1 40 20 Z",
        "M 0 0 Q 20 40 40 0 T 80 0",
        "M 0 0 C 10 -30 30 30 40 0 S 70 -30 80 0 Z M 30 -5 h 10 v 10 h -10 Z",
        "M 10 10 L 30 10 L 20 30 Z M 50 50 L 60 50 L 55 60 Z M 0 60 h 5 v 5 Z",
        "M 0 0 A 30 10 20 1 1 40 20",
    ];

    #[test]
    fn the_ends_are_the_paths() {
        for from in SHAPES {
            for to in SHAPES {
                let morph = Morph::new(parse(from), parse(to));
                let (start, end) = (morph.at(0.0), morph.at(1.0));
                // Subpaths added to grow from a point draw nothing at their end
                let drawn = |path: &[PathSegment]| -> Vec<PathSegment> {
                    split_subpaths(path)
                        .into_iter()
                        .filter(|s| PathMeasure::new(s).total_length() > 1e-9)
                        .flatten()
                        .collect()
                };
                assert!(
                    distance(&drawn(&start), &parse(from)) < 1e-3,
                    "{from} -> {to}"
                );
                assert!(distance(&drawn(&end), &parse(to)) < 1e-3, "{from} -> {to}");
            }
        }
    }

    #[test]
    fn every_step_has_the_same_segments() {
        for (from, to) in [
            (SHAPES[0], SHAPES[1]),
            (SHAPES[3], SHAPES[4]),
            (SHAPES[2], SHAPES[5]),
        ] {
            let morph = Morph::new(parse(from), parse(to));
            let kinds = |t: f64| -> Vec<std::mem::Discriminant<PathSegment>> {
                morph.at(t).iter().map(std::mem::discriminant).collect()
            };
            for t in [0.25, 0.5, 0.75, 1.0] {
                assert_eq!(kinds(0.0), kinds(t));
            }
        }
    }

    #[test]
    fn a_shape_does_not_twist_on_its_way() {
        let square = parse("M 0 0 H 20 V 20 H 0 Z");
        // The same square, started elsewhere and drawn the other way
        let turned = parse("M 20 20 V 0 H 0 V 20 Z");
        let halfway = interpolate(&square, &turned, 0.5);
        assert!(distance(&halfway, &square) < 1e-9);
    }

    #[test]
    fn a_subpath_without_a_partner_grows_from_a_point() {
        let one = parse("M 0 0 H 20 V 20 H 0 Z");
        let two = parse("M 0 0 H 20 V 20 H 0 Z M 40 0 H 60 V 20 H 40 Z");
        let morph = Morph::new(&one, &two);
        let area = |t: f64| PathMeasure::new(morph.at(t)).area();
        assert!((area(0.0) - 400.0).abs() < 1e-9);
        assert!((area(0.5) - 500.0).abs() < 1e-9);
        assert!((area(1.0) - 800.0).abs() < 1e-9);
    }

    #[test]
    fn open_and_closed_subpaths() {
        let open = parse("M 0 0 L 10 0 L 10 10");
        let closed = parse("M 0 0 L 10 0 L 10 10 Z");
        let path = interpolate(&open, &closed, 0.5);
        assert!(!path
            .iter()
            .any(|s| matches!(s, PathSegment::ClosePath { .. })));
        let both = interpolate(&closed, &closed, 0.5);
        assert!(matches!(both.last(), Some(PathSegment::ClosePath { .. })));
    }

    #[test]
    fn nothing_to_nothing() {
        let none: Vec<PathSegment> = Vec::new();
        assert!(interpolate(&none, &none, 0.5).is_empty());
    }
}
