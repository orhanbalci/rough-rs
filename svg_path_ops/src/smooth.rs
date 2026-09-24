use kurbo::{CubicBez, Point, Vec2};
use svgtypes::PathSegment;

use crate::measure::{corner_runs, PathMeasure, Piece};

/// How [`PathMeasure::smooth`] draws curves through a path's points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Smoothing {
    /// A cubic spline: the curves meet with the same direction and the same
    /// curvature at every point, so the path bends without a visible seam
    /// anywhere. Moving one point reshapes the whole stretch around it.
    Continuous,
    /// A Catmull–Rom spline: each point's direction comes from its two
    /// neighbours only, so moving one point reshapes just the curves next
    /// to it. `alpha` spaces the points' parameters by their distance
    /// raised to it: 0 is the uniform spline, 0.5 the centripetal one,
    /// which never loops or cusps within a curve, and 1 the chordal one.
    CatmullRom { alpha: f64 },
}

impl PathMeasure {
    /// The path redrawn as smooth curves through the points where its
    /// segments meet: a polygon becomes a rounded shape through its
    /// corners, and a path of lines a flowing curve through their ends.
    ///
    /// Joins that turn by more than `corner_angle` degrees stay corners;
    /// 180 smooths every join. Between corners every segment becomes a
    /// cubic curve through the same two ends, except a stretch of a single
    /// segment, which stays as it is. Subpaths stay apart, and a closed
    /// subpath stays closed; where it has no corner at its start, it is
    /// smoothed across that point too. The result is in absolute
    /// coordinates.
    ///
    /// Only the points where segments meet shape the result, as in
    /// Paper.js's `smooth`: the curves and arcs of the path are replaced,
    /// not followed.
    ///
    /// # Algorithm
    ///
    /// Both kinds find a direction and speed for the curve at every point
    /// and turn them into the control points of cubic Bézier curves: a
    /// curve leaving point `P` with velocity `D` over a parameter span `h`
    /// has its first control point at `P + D h / 3`.
    ///
    /// [`Smoothing::Continuous`] is the cubic spline with chord length
    /// parameters: the span between two points is their distance, so
    /// unevenly spaced points do not make the curve overshoot. Asking the
    /// second derivative to match where curves meet gives one equation per
    /// point in the velocities there,
    ///
    /// ```text
    /// hᵢ Dᵢ₋₁ + 2 (hᵢ₋₁ + hᵢ) Dᵢ + hᵢ₋₁ Dᵢ₊₁ = 3 (hᵢ/hᵢ₋₁ (Pᵢ − Pᵢ₋₁) + hᵢ₋₁/hᵢ (Pᵢ₊₁ − Pᵢ))
    /// ```
    ///
    /// with `hᵢ` the distance from `Pᵢ` to `Pᵢ₊₁`. An open stretch adds
    /// the natural end conditions, no curvature at its ends, and is solved
    /// as a tridiagonal system with the Thomas algorithm. A closed subpath
    /// wraps the equations around into a cyclic tridiagonal system, solved
    /// with the Sherman–Morrison formula; either takes time linear in the
    /// number of points.
    ///
    /// [`Smoothing::CatmullRom`] gives the curve from `P₁` to `P₂` the
    /// control points of the Catmull–Rom curve through `P₀`, `P₁`, `P₂`,
    /// `P₃` with parameters spaced by `dᵢ = |Pᵢ − Pᵢ₋₁|^alpha`, in the
    /// closed form of Yuksel, Schaefer and Keyser's *Parameterization and
    /// Applications of Catmull–Rom Curves* (2011):
    ///
    /// ```text
    /// B₁ = (d₁² P₂ − d₂² P₀ + (2d₁² + 3d₁d₂ + d₂²) P₁) / (3d₁ (d₁ + d₂))
    /// B₂ = (d₃² P₁ − d₂² P₃ + (2d₃² + 3d₃d₂ + d₂²) P₂) / (3d₃ (d₃ + d₂))
    /// ```
    ///
    /// At the ends of an open stretch, the missing neighbour is the next
    /// point mirrored through the end, so the curve leaves along the line
    /// to the next point.
    ///
    /// ```
    /// use svg_path_ops::svgtypes::PathParser;
    /// use svg_path_ops::{PathMeasure, PathSegment, Smoothing};
    ///
    /// let square: Vec<_> = PathParser::from("M 0 0 H 100 V 100 H 0 Z").collect::<Result<_, _>>()?;
    /// let rounded = PathMeasure::new(&square).smooth(Smoothing::Continuous, 180.0);
    ///
    /// // Four curves through the corners, and a close path
    /// assert_eq!(rounded.len(), 6);
    /// assert!(matches!(
    ///     rounded[2],
    ///     PathSegment::CurveTo { x: 100.0, y: 100.0, .. }
    /// ));
    /// # Ok::<(), svg_path_ops::svgtypes::Error>(())
    /// ```
    pub fn smooth(&self, smoothing: Smoothing, corner_angle: f64) -> Vec<PathSegment> {
        let corner = corner_angle.to_radians();
        let mut path = Vec::new();
        for subpath in self.subpaths() {
            let start = subpath[0].from;
            path.push(PathSegment::MoveTo { abs: true, x: start.x, y: start.y });
            let closed = subpath.last().is_some_and(|piece| piece.closes);
            let drawing: Vec<&Piece> = subpath.iter().filter(|p| p.length > 0.0).collect();
            let mut runs: Vec<Vec<usize>> = corner_runs(&drawing, corner)
                .into_iter()
                .map(|run| run.collect())
                .collect();

            // A closed subpath with no corner at its start is smoothed
            // across it: all round when it has no corners at all, otherwise
            // its last stretch goes on into its first
            let n = drawing.len();
            let seam = closed && n > 1 && drawing[n - 1].turn_to(drawing[0]) <= corner;
            let mut cyclic = false;
            if seam {
                if runs.len() == 1 {
                    cyclic = true;
                } else {
                    let first = runs.remove(0);
                    runs.last_mut().expect("runs remain").extend(first);
                }
            }

            // The curve each piece of `drawing` becomes, or `None` to keep it
            let mut curves: Vec<Option<CubicBez>> = vec![None; n];
            for run in &runs {
                if run.len() < 2 {
                    continue;
                }
                // The points the curves pass through: where each piece
                // starts, and where the last ends unless the run goes all
                // round, where that is the first point again
                let mut points: Vec<Point> = run.iter().map(|&i| drawing[i].from).collect();
                if !cyclic {
                    points.push(drawing[run[run.len() - 1]].to);
                }
                // Two points in the same place have no direction between
                // them; such a stretch keeps its segments
                let span = |i: usize| (points[(i + 1) % points.len()] - points[i]).hypot();
                if (0..run.len()).any(|i| span(i) <= 1e-12 * (1.0 + points[i].to_vec2().hypot())) {
                    continue;
                }
                let fitted = match smoothing {
                    Smoothing::Continuous => continuous(&points, cyclic),
                    Smoothing::CatmullRom { alpha } => catmull_rom(&points, cyclic, alpha),
                };
                for (&i, curve) in run.iter().zip(fitted) {
                    curves[i] = Some(curve);
                }
            }

            for (piece, curve) in drawing.iter().zip(curves) {
                path.push(match curve {
                    Some(c) => PathSegment::CurveTo {
                        abs: true,
                        x1: c.p1.x,
                        y1: c.p1.y,
                        x2: c.p2.x,
                        y2: c.p2.y,
                        x: c.p3.x,
                        y: c.p3.y,
                    },
                    None => piece.as_given(),
                });
            }
            if closed && !matches!(path.last(), Some(PathSegment::ClosePath { .. })) {
                path.push(PathSegment::ClosePath { abs: true });
            }
        }
        path
    }
}

/// The cubic spline through `points` with chord length parameters, as one
/// curve between each point and the next, and from the last back to the
/// first when `cyclic`.
fn continuous(points: &[Point], cyclic: bool) -> Vec<CubicBez> {
    let count = points.len();
    let segments = if cyclic { count } else { count - 1 };
    let point = |i: usize| points[i % count];
    let h: Vec<f64> = (0..segments)
        .map(|i| (point(i + 1) - point(i)).hypot())
        .collect();

    // The velocity at each point, from one equation per point
    let velocities = if cyclic {
        let (mut sub, mut diag, mut sup, mut rhs) = (vec![], vec![], vec![], vec![]);
        for i in 0..count {
            let (before, after) = (h[(i + count - 1) % count], h[i]);
            sub.push(after);
            diag.push(2.0 * (before + after));
            sup.push(before);
            let into = point(i) - point(i + count - 1);
            let out = point(i + 1) - point(i);
            rhs.push((into * (after / before) + out * (before / after)) * 3.0);
        }
        solve_cyclic(&sub, &diag, &sup, rhs)
    } else {
        let (mut sub, mut diag, mut sup, mut rhs) = (vec![0.0], vec![2.0], vec![1.0], vec![]);
        rhs.push((point(1) - point(0)) * (3.0 / h[0]));
        for i in 1..count - 1 {
            let (before, after) = (h[i - 1], h[i]);
            sub.push(after);
            diag.push(2.0 * (before + after));
            sup.push(before);
            let into = point(i) - point(i - 1);
            let out = point(i + 1) - point(i);
            rhs.push((into * (after / before) + out * (before / after)) * 3.0);
        }
        // The natural end: no curvature at the last point
        sub.push(1.0);
        diag.push(2.0);
        sup.push(0.0);
        rhs.push((point(count - 1) - point(count - 2)) * (3.0 / h[count - 2]));
        solve_tridiagonal(&sub, &diag, &sup, rhs)
    };

    (0..segments)
        .map(|i| {
            let (p0, p3) = (point(i), point(i + 1));
            let third = h[i] / 3.0;
            CubicBez::new(
                p0,
                p0 + velocities[i] * third,
                p3 - velocities[(i + 1) % count] * third,
                p3,
            )
        })
        .collect()
}

/// Solves the tridiagonal system with `sub`, `diag` and `sup` on its
/// diagonals for `rhs`, with the Thomas algorithm. The system must be
/// diagonally dominant, as the spline's is.
fn solve_tridiagonal(sub: &[f64], diag: &[f64], sup: &[f64], mut rhs: Vec<Vec2>) -> Vec<Vec2> {
    let n = diag.len();
    let mut sup_prime = vec![0.0; n];
    sup_prime[0] = sup[0] / diag[0];
    rhs[0] /= diag[0];
    for i in 1..n {
        let m = diag[i] - sub[i] * sup_prime[i - 1];
        sup_prime[i] = sup[i] / m;
        rhs[i] = (rhs[i] - rhs[i - 1] * sub[i]) / m;
    }
    for i in (0..n - 1).rev() {
        rhs[i] = rhs[i] - rhs[i + 1] * sup_prime[i];
    }
    rhs
}

/// Solves the tridiagonal system whose first row also has `sub[0]` in its
/// last column and whose last row has `sup[n − 1]` in its first, with the
/// Sherman–Morrison formula over two tridiagonal solves.
fn solve_cyclic(sub: &[f64], diag: &[f64], sup: &[f64], rhs: Vec<Vec2>) -> Vec<Vec2> {
    let n = diag.len();
    if n == 2 {
        // Both corners of the matrix sit next to the diagonal
        let (a, b) = (sub[0] + sup[0], sub[1] + sup[1]);
        let det = diag[0] * diag[1] - a * b;
        return vec![
            (rhs[0] * diag[1] - rhs[1] * a) / det,
            (rhs[1] * diag[0] - rhs[0] * b) / det,
        ];
    }
    // A = B + u vᵀ with u = (γ, 0, …, 0, sup[n−1]) and v = (1, 0, …, 0,
    // sub[0] / γ), where B is tridiagonal
    let (top_right, bottom_left) = (sub[0], sup[n - 1]);
    let gamma = -diag[0];
    let mut modified = diag.to_vec();
    modified[0] -= gamma;
    modified[n - 1] -= bottom_left * top_right / gamma;
    let (mut sub, mut sup) = (sub.to_vec(), sup.to_vec());
    sub[0] = 0.0;
    sup[n - 1] = 0.0;

    let x = solve_tridiagonal(&sub, &modified, &sup, rhs);
    let mut u = vec![Vec2::ZERO; n];
    u[0] = Vec2::new(gamma, gamma);
    u[n - 1] = Vec2::new(bottom_left, bottom_left);
    let z = solve_tridiagonal(&sub, &modified, &sup, u);
    // Each coordinate is its own system with the same matrix
    let factor = |x0: f64, xn: f64, z0: f64, zn: f64| {
        (x0 + top_right * xn / gamma) / (1.0 + z0 + top_right * zn / gamma)
    };
    let fx = factor(x[0].x, x[n - 1].x, z[0].x, z[n - 1].x);
    let fy = factor(x[0].y, x[n - 1].y, z[0].y, z[n - 1].y);
    x.iter()
        .zip(&z)
        .map(|(x, z)| Vec2::new(x.x - fx * z.x, x.y - fy * z.y))
        .collect()
}

/// The Catmull–Rom spline through `points` with parameters spaced by the
/// distances raised to `alpha`, as one curve between each point and the
/// next, and from the last back to the first when `cyclic`.
fn catmull_rom(points: &[Point], cyclic: bool, alpha: f64) -> Vec<CubicBez> {
    let alpha = alpha.clamp(0.0, 1.0);
    let count = points.len() as isize;
    let segments = if cyclic { count } else { count - 1 };
    // Points before the first and after the last mirror their neighbour
    let point = |i: isize| {
        if cyclic {
            points[i.rem_euclid(count) as usize]
        } else if i < 0 {
            points[0] + (points[0] - points[1])
        } else if i >= count {
            let last = points[count as usize - 1];
            last + (last - points[count as usize - 2])
        } else {
            points[i as usize]
        }
    };
    let spacing = |a: Point, b: Point| (b - a).hypot().powf(alpha);

    (0..segments)
        .map(|i| {
            let (p0, p1, p2, p3) = (point(i - 1), point(i), point(i + 1), point(i + 2));
            let (d1, d2, d3) = (spacing(p0, p1), spacing(p1, p2), spacing(p2, p3));
            let b1 = (p2.to_vec2() * (d1 * d1) - p0.to_vec2() * (d2 * d2)
                + p1.to_vec2() * (2.0 * d1 * d1 + 3.0 * d1 * d2 + d2 * d2))
                / (3.0 * d1 * (d1 + d2));
            let b2 = (p1.to_vec2() * (d3 * d3) - p3.to_vec2() * (d2 * d2)
                + p2.to_vec2() * (2.0 * d3 * d3 + 3.0 * d3 * d2 + d2 * d2))
                / (3.0 * d3 * (d3 + d2));
            CubicBez::new(p1, b1.to_point(), b2.to_point(), p2)
        })
        .collect()
}

#[cfg(test)]
mod test {
    use kurbo::{CubicBez, ParamCurve, ParamCurveDeriv, Point};
    use svgtypes::{PathParser, PathSegment};

    use super::Smoothing;
    use crate::write::{write_path, WriteOptions};
    use crate::PathMeasure;

    const CENTRIPETAL: Smoothing = Smoothing::CatmullRom { alpha: 0.5 };

    fn smooth(path: &str, smoothing: Smoothing, corner_angle: f64) -> Vec<PathSegment> {
        let segments: Vec<PathSegment> = PathParser::from(path).collect::<Result<_, _>>().unwrap();
        PathMeasure::new(segments).smooth(smoothing, corner_angle)
    }

    fn written(segments: &[PathSegment]) -> String {
        write_path(segments, &WriteOptions::default())
    }

    /// The curves in `segments`, which start with a move.
    fn curves(segments: &[PathSegment]) -> Vec<CubicBez> {
        let mut at = Point::ZERO;
        let mut curves = Vec::new();
        for segment in segments {
            match *segment {
                PathSegment::MoveTo { x, y, .. } | PathSegment::LineTo { x, y, .. } => {
                    at = Point::new(x, y)
                }
                PathSegment::CurveTo { x1, y1, x2, y2, x, y, .. } => {
                    let end = Point::new(x, y);
                    curves.push(CubicBez::new(
                        at,
                        Point::new(x1, y1),
                        Point::new(x2, y2),
                        end,
                    ));
                    at = end;
                }
                _ => {}
            }
        }
        curves
    }

    fn curvature(c: &CubicBez, t: f64) -> f64 {
        let d1 = c.deriv().eval(t).to_vec2();
        let d2 = c.deriv().deriv().eval(t).to_vec2();
        d1.cross(d2) / d1.hypot().powi(3)
    }

    fn direction(c: &CubicBez, t: f64) -> kurbo::Vec2 {
        let d = c.deriv().eval(t).to_vec2();
        d / d.hypot()
    }

    /// Points unevenly spaced along a wave.
    const WAVE: &str = "M 0 0 L 10 20 L 40 25 L 45 0 L 90 -30 L 100 0";
    /// A closed polygon with sides of different lengths.
    const BLOB: &str = "M 0 0 L 60 -10 L 100 30 L 70 90 L 20 70 Z";

    #[test]
    fn passes_through_the_points() {
        for smoothing in [Smoothing::Continuous, CENTRIPETAL] {
            let smoothed = curves(&smooth(WAVE, smoothing, 180.0));
            let ends: Vec<_> = smoothed.iter().map(|c| (c.p3.x, c.p3.y)).collect();
            assert_eq!(
                ends,
                [
                    (10.0, 20.0),
                    (40.0, 25.0),
                    (45.0, 0.0),
                    (90.0, -30.0),
                    (100.0, 0.0)
                ]
            );
        }
    }

    #[test]
    fn continuous_keeps_the_curvature_where_curves_meet() {
        for (path, closed) in [(WAVE, false), (BLOB, true)] {
            let smoothed = curves(&smooth(path, Smoothing::Continuous, 180.0));
            let mut pairs: Vec<_> = smoothed.windows(2).map(|w| (w[0], w[1])).collect();
            if closed {
                pairs.push((smoothed[smoothed.len() - 1], smoothed[0]));
            }
            for (a, b) in pairs {
                assert!((direction(&a, 1.0) - direction(&b, 0.0)).hypot() < 1e-9);
                let (ka, kb) = (curvature(&a, 1.0), curvature(&b, 0.0));
                assert!((ka - kb).abs() < 1e-9 * (1.0 + ka.abs()), "{ka} {kb}");
            }
        }
    }

    #[test]
    fn continuous_has_no_curvature_at_open_ends() {
        let smoothed = curves(&smooth(WAVE, Smoothing::Continuous, 180.0));
        assert!(curvature(&smoothed[0], 0.0).abs() < 1e-12);
        assert!(curvature(&smoothed[smoothed.len() - 1], 1.0).abs() < 1e-12);
    }

    #[test]
    fn catmull_rom_keeps_the_direction_where_curves_meet() {
        for alpha in [0.0, 0.5, 1.0] {
            let smoothing = Smoothing::CatmullRom { alpha };
            let smoothed = curves(&smooth(BLOB, smoothing, 180.0));
            let n = smoothed.len();
            for i in 0..n {
                let (a, b) = (smoothed[i], smoothed[(i + 1) % n]);
                assert!((direction(&a, 1.0) - direction(&b, 0.0)).hypot() < 1e-9);
            }
        }
    }

    #[test]
    fn uniform_catmull_rom_controls() {
        // The tangent at a point is half the step between its neighbours
        let smoothed = curves(&smooth(
            "M 0 0 L 12 0 L 24 12",
            Smoothing::CatmullRom { alpha: 0.0 },
            180.0,
        ));
        assert_eq!(smoothed[1].p1, Point::new(12.0 + 24.0 / 6.0, 12.0 / 6.0));
    }

    #[test]
    fn a_regular_polygon_becomes_nearly_a_circle() {
        let mut path = String::from("M 100 0");
        for i in 1..12 {
            let angle = f64::from(i) / 12.0 * std::f64::consts::TAU;
            path += &format!(" L {} {}", 100.0 * angle.cos(), 100.0 * angle.sin());
        }
        path += " Z";
        for smoothing in [Smoothing::Continuous, CENTRIPETAL] {
            let smoothed = smooth(&path, smoothing, 180.0);
            assert!(matches!(
                smoothed.last(),
                Some(PathSegment::ClosePath { .. })
            ));
            for c in curves(&smoothed) {
                for k in 0..=10 {
                    let radius = c.eval(f64::from(k) / 10.0).to_vec2().hypot();
                    assert!((radius - 100.0).abs() < 0.2, "{radius}");
                }
            }
        }
    }

    #[test]
    fn points_on_a_line_stay_on_it() {
        let smoothed = curves(&smooth(
            "M 0 0 L 10 0 L 40 0 L 45 0",
            Smoothing::Continuous,
            180.0,
        ));
        for c in smoothed {
            assert_eq!([c.p1.y, c.p2.y], [0.0, 0.0]);
            assert!(c.p0.x <= c.p1.x && c.p1.x <= c.p2.x && c.p2.x <= c.p3.x);
        }
    }

    #[test]
    fn keeps_corners() {
        let path = "M 0 0 L 50 10 L 100 0 L 100 100";
        let smoothed = smooth(path, Smoothing::Continuous, 60.0);
        assert!(matches!(smoothed[1], PathSegment::CurveTo { .. }));
        assert!(matches!(smoothed[2], PathSegment::CurveTo { .. }));
        assert!(written(&smoothed).ends_with(" L 100 100"));
        // The corner at 100 0 stays sharp
        let c = curves(&smoothed);
        assert!(direction(&c[1], 1.0).dot(kurbo::Vec2::new(0.0, 1.0)) < 0.5);
    }

    #[test]
    fn smooths_across_the_start_of_a_closed_path() {
        // Corners at the far ends; the start is in the middle of a gently
        // bent side, and the curves run on through it
        let path = "M 50 0 L 100 5 L 150 0 L 150 100 L -50 100 L -50 0 L 0 5 Z";
        let smoothed = curves(&smooth(path, Smoothing::Continuous, 60.0));
        let (first, last) = (smoothed[0], smoothed[smoothed.len() - 1]);
        assert_eq!(last.p3, Point::new(50.0, 0.0));
        assert!((direction(&last, 1.0) - direction(&first, 0.0)).hypot() < 1e-9);
    }

    #[test]
    fn keeps_single_segments_and_subpaths() {
        let path = "M 0 0 A 10 10 0 0 1 20 0 M 30 0 L 40 0 Z M 50 0 L 60 5 L 70 0";
        let smoothed = smooth(path, Smoothing::Continuous, 60.0);
        let text = written(&smoothed);
        assert!(
            text.starts_with("M 0 0 A 10 10 0 0 1 20 0 M 30 0 "),
            "{text}"
        );
        assert_eq!(
            smoothed
                .iter()
                .filter(|s| matches!(s, PathSegment::MoveTo { .. }))
                .count(),
            3
        );
    }

    #[test]
    fn a_path_that_draws_nothing() {
        assert!(smooth("", Smoothing::Continuous, 180.0).is_empty());
        assert_eq!(written(&smooth("M 5 5 Z", CENTRIPETAL, 180.0)), "M 5 5 Z");
    }
}
