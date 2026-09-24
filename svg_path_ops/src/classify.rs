use euclid::default::Point2D;
use kurbo::Vec2;

use crate::measure::{PathMeasure, PieceShape};

/// The shape of a segment, as [`PathMeasure::classify`] tells it, after
/// Paper.js's `classify`. Parameters are the segment's own, from 0 at its
/// start to 1 at its end, and only features strictly inside it count.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CurveKind {
    /// A straight line, or a curve whose points all lie on one.
    Line,
    /// A quadratic curve, or a cubic one that is really quadratic: a piece
    /// of a parabola.
    Quadratic,
    /// A curve that bends one way only, with no inflection, cusp or loop.
    /// Arcs are arches.
    Arch,
    /// An S-shaped curve, which bends one way and then the other, at one or
    /// two inflections.
    Serpentine { first: f64, second: Option<f64> },
    /// A curve that comes to a point, where it stops and turns back.
    Cusp { at: f64 },
    /// A curve that crosses itself, at parameters `first` and `second`.
    Loop { first: f64, second: f64 },
}

impl CurveKind {
    /// The kind of the cubic Bézier curve with control points `points`.
    ///
    /// ```
    /// use svg_path_ops::euclid::default::Point2D;
    /// use svg_path_ops::CurveKind;
    ///
    /// let p = |x, y| Point2D::new(x, y);
    /// let kind = CurveKind::of_cubic([p(0.0, 0.0), p(40.0, 30.0), p(-10.0, 30.0), p(30.0, 0.0)]);
    /// assert!(matches!(kind, CurveKind::Loop { .. }));
    /// ```
    ///
    /// # Algorithm
    ///
    /// Written as `B(t) = P₀ + c₁t + c₂t² + c₃t³`, the curve turns the
    /// way the sign of `B′(t) × B″(t)` says, and that cross product is
    /// `2 (3k₂₃t² + 3k₁₃t + k₁₂)` with `kᵢⱼ = cᵢ × cⱼ`. How this quadratic's
    /// roots fall tells the kind, as Stone and DeRose's *A Geometric
    /// Characterization of Parametric Cubic Curves* (1989) shows: with
    /// discriminant `D = 9k₁₃² − 12k₂₃k₁₂`, two real roots (`D > 0`) are
    /// inflections, a double root (`D = 0`) is a cusp, and no real roots
    /// (`D < 0`) mean the curve, run on past its ends, crosses itself.
    ///
    /// Where it crosses itself, `B(s) = B(u)` with `s ≠ u`; dividing
    /// `B(s) − B(u)` by `s − u` leaves `c₁ + c₂σ + c₃(σ² − π) = 0` in
    /// `σ = s + u` and `π = su`. Its cross product with `c₃` gives
    /// `σ = −k₁₃ / k₂₃`, its dot product with `c₃` then gives `π`, and `s`
    /// and `u` are the roots of `z² − σz + π`.
    ///
    /// When `c₃` is zero the curve is quadratic; when `k₂₃` is zero there is
    /// at most one inflection, at `−k₁₂ / 3k₁₃`, and when every `kᵢⱼ` is zero
    /// the curve is straight. A feature outside the segment, or a loop that
    /// does not close inside it, leaves an arch, as in Paper.js.
    pub fn of_cubic(points: [Point2D<f64>; 4]) -> CurveKind {
        let [p0, p1, p2, p3] = points.map(|p| Vec2::new(p.x, p.y));
        let c1 = (p1 - p0) * 3.0;
        let c2 = (p0 - p1 * 2.0 + p2) * 3.0;
        let c3 = p3 - p0 + (p1 - p2) * 3.0;
        let (k12, k13, k23) = (c1.cross(c2), c1.cross(c3), c2.cross(c3));

        // What counts as zero, by the size of the curve
        let size = c1.hypot().max(c2.hypot()).max(c3.hypot());
        let zero = |k: f64| k.abs() <= 1e-12 * size * size;
        let inside = |t: f64| t > 0.0 && t < 1.0;

        if zero(k12) && zero(k13) && zero(k23) {
            return CurveKind::Line;
        }
        if c3.hypot() <= 1e-12 * size {
            return CurveKind::Quadratic;
        }
        if zero(k23) {
            // One inflection at most
            if zero(k13) {
                return CurveKind::Arch;
            }
            let t = -k12 / (3.0 * k13);
            return if inside(t) {
                CurveKind::Serpentine { first: t, second: None }
            } else {
                CurveKind::Arch
            };
        }

        let d = 9.0 * k13 * k13 - 12.0 * k23 * k12;
        let scale = 9.0 * k13 * k13 + 12.0 * (k23 * k12).abs();
        if d.abs() <= 1e-9 * scale {
            let at = -k13 / (2.0 * k23);
            return if inside(at) {
                CurveKind::Cusp { at }
            } else {
                CurveKind::Arch
            };
        }
        if d > 0.0 {
            let root = d.sqrt();
            let (a, b) = (
                (-3.0 * k13 - root) / (6.0 * k23),
                (-3.0 * k13 + root) / (6.0 * k23),
            );
            let mut roots: Vec<f64> = [a.min(b), a.max(b)]
                .into_iter()
                .filter(|t| inside(*t))
                .collect();
            return match roots.len() {
                0 => CurveKind::Arch,
                1 => CurveKind::Serpentine { first: roots[0], second: None },
                _ => {
                    let second = roots.pop();
                    CurveKind::Serpentine { first: roots[0], second }
                }
            };
        }

        // Where it crosses itself
        let sigma = -k13 / k23;
        let pi = sigma * sigma + (c1 + c2 * sigma).dot(c3) / c3.hypot2();
        let disc = (sigma * sigma - 4.0 * pi).max(0.0).sqrt();
        let (first, second) = ((sigma - disc) / 2.0, (sigma + disc) / 2.0);
        if inside(first) && inside(second) {
            CurveKind::Loop { first, second }
        } else {
            CurveKind::Arch
        }
    }
}

impl PathMeasure {
    /// The kind of the segment at `index` in the path: a line, a quadratic
    /// curve, an arch that bends one way, or a cubic curve with inflections,
    /// a cusp or a loop inside it. See [`CurveKind::of_cubic`] for how
    /// cubic curves are told apart. `None` for a move, a segment that draws
    /// nothing, or an index past the end.
    ///
    /// ```
    /// use svg_path_ops::pt::PathTransformer;
    /// use svg_path_ops::CurveKind;
    ///
    /// let measure = PathTransformer::parse("M 0 0 C 10 -10 20 10 30 0 L 40 0")?.measure();
    /// assert!(matches!(
    ///     measure.classify(1),
    ///     Some(CurveKind::Serpentine { second: None, .. })
    /// ));
    /// assert_eq!(measure.classify(2), Some(CurveKind::Line));
    /// assert_eq!(measure.classify(0), None);
    /// # Ok::<(), svg_path_ops::svgtypes::Error>(())
    /// ```
    pub fn classify(&self, index: usize) -> Option<CurveKind> {
        let piece = self.pieces.iter().find(|piece| piece.index == index)?;
        if piece.length == 0.0 {
            return None;
        }
        let point = |p: kurbo::Point| Point2D::new(p.x, p.y);
        Some(match piece.shape {
            PieceShape::Line(_) => CurveKind::Line,
            PieceShape::Quadratic(quad) => {
                let [a, b, c] = [quad.p0, quad.p1, quad.p2];
                if (b - a).cross(c - a) == 0.0 {
                    CurveKind::Line
                } else {
                    CurveKind::Quadratic
                }
            }
            PieceShape::Cubic(cubic) => {
                CurveKind::of_cubic([cubic.p0, cubic.p1, cubic.p2, cubic.p3].map(point))
            }
            PieceShape::Arc(_) => CurveKind::Arch,
        })
    }
}

#[cfg(test)]
mod test {
    use euclid::default::Point2D;
    use kurbo::{CubicBez, ParamCurve, ParamCurveDeriv, Point};
    use svgtypes::{PathParser, PathSegment};

    use super::CurveKind;
    use crate::PathMeasure;

    fn kind(points: [(f64, f64); 4]) -> CurveKind {
        CurveKind::of_cubic(points.map(|(x, y)| Point2D::new(x, y)))
    }

    fn cubic(points: [(f64, f64); 4]) -> CubicBez {
        let [a, b, c, d] = points.map(|(x, y)| Point::new(x, y));
        CubicBez::new(a, b, c, d)
    }

    /// The sign of the way the curve turns at `t`.
    fn turning(curve: &CubicBez, t: f64) -> f64 {
        let d1 = curve.deriv().eval(t).to_vec2();
        let d2 = curve.deriv().deriv().eval(t).to_vec2();
        d1.cross(d2).signum()
    }

    #[test]
    fn serpentines_turn_the_other_way_at_their_inflections() {
        for points in [
            [(0.0, 0.0), (10.0, -10.0), (20.0, 10.0), (30.0, 0.0)],
            [(0.0, 0.0), (30.0, -40.0), (-5.0, 35.0), (40.0, 5.0)],
            [(0.0, 0.0), (5.0, 20.0), (25.0, -20.0), (27.0, 30.0)],
        ] {
            let curve = cubic(points);
            let CurveKind::Serpentine { first, second } = kind(points) else {
                panic!("{points:?} is {:?}", kind(points));
            };
            for t in std::iter::once(first).chain(second) {
                assert!(t > 0.0 && t < 1.0);
                assert_ne!(turning(&curve, t - 1e-6), turning(&curve, t + 1e-6));
            }
            if let Some(second) = second {
                assert!(first < second);
            }
        }
    }

    #[test]
    fn loops_cross_themselves_where_they_say() {
        for points in [
            [(0.0, 0.0), (40.0, 30.0), (-10.0, 30.0), (30.0, 0.0)],
            [(0.0, 0.0), (100.0, 100.0), (-50.0, 100.0), (50.0, 0.0)],
        ] {
            let CurveKind::Loop { first, second } = kind(points) else {
                panic!("{points:?} is {:?}", kind(points));
            };
            let curve = cubic(points);
            assert!(first < second);
            assert!((curve.eval(first) - curve.eval(second)).hypot() < 1e-9);
        }
    }

    #[test]
    fn cusps_stop_where_they_say() {
        // Control points crossed so that the curve stops and turns at 0.5
        let points = [(0.0, 0.0), (30.0, 30.0), (0.0, 30.0), (30.0, 0.0)];
        let CurveKind::Cusp { at } = kind(points) else {
            panic!("{:?}", kind(points));
        };
        assert!((at - 0.5).abs() < 1e-6);
        assert!(cubic(points).deriv().eval(at).to_vec2().hypot() < 1e-6);
    }

    #[test]
    fn arches_lines_and_quadratics() {
        assert_eq!(
            kind([(0.0, 0.0), (0.0, 10.0), (20.0, 10.0), (20.0, 0.0)]),
            CurveKind::Arch
        );
        assert_eq!(
            kind([(0.0, 0.0), (5.0, 5.0), (20.0, 20.0), (7.0, 7.0)]),
            CurveKind::Line
        );
        // A quadratic curve raised to a cubic
        assert_eq!(
            kind([(0.0, 0.0), (20.0, 40.0), (40.0, 40.0), (60.0, 0.0)]),
            CurveKind::Quadratic
        );
        // A loop that closes past the end is an arch
        assert_eq!(
            kind([(0.0, 0.0), (40.0, 30.0), (-10.0, 30.0), (0.0, 25.0)]),
            CurveKind::Arch
        );
        // A control point on the end is a cusp at the end: an arch
        assert_eq!(
            kind([(0.0, 0.0), (0.0, 0.0), (20.0, 10.0), (30.0, 0.0)]),
            CurveKind::Arch
        );
    }

    #[test]
    fn classifies_the_segments_of_a_path() {
        let segments: Vec<PathSegment> =
            PathParser::from("M 0 0 h 10 q 5 5 10 0 t 10 0 a 5 5 0 0 1 10 0 s 5 -10 10 0 z")
                .collect::<Result<_, _>>()
                .unwrap();
        let measure = PathMeasure::new(&segments);
        let kinds: Vec<_> = (0..=7).map(|i| measure.classify(i)).collect();
        assert_eq!(
            kinds,
            [
                None,
                Some(CurveKind::Line),
                Some(CurveKind::Quadratic),
                Some(CurveKind::Quadratic),
                Some(CurveKind::Arch),
                Some(CurveKind::Arch),
                Some(CurveKind::Line),
                None
            ]
        );
    }
}
