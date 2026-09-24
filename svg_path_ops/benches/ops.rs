//! Benchmarks of the main operations on Ferris the crab, a path of 135
//! segments in eight subpaths, and on a few simpler shapes.
//!
//! ```sh
//! cargo bench -p svg_path_ops
//! ```

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use svg_path_ops::euclid::default::Point2D;
use svg_path_ops::pt::PathTransformer;
use svg_path_ops::svgtypes::{PathParser, PathSegment};
use svg_path_ops::{
    boolean,
    optimize,
    write_path,
    BooleanOp,
    BooleanOptions,
    FillRule,
    LineJoin,
    Morph,
    PathMeasure,
    StrokeStyle,
    WriteOptions,
};

const FERRIS: &str = include_str!("ferris.txt");
const CIRCLE: &str = "M 900 450 A 300 300 0 0 1 300 450 A 300 300 0 0 1 900 450 Z";

fn parse(path: &str) -> Vec<PathSegment> {
    PathParser::from(path).collect::<Result<_, _>>().unwrap()
}

fn text(c: &mut Criterion) {
    let ferris = parse(FERRIS);
    c.bench_function("parse", |b| b.iter(|| parse(black_box(FERRIS))));
    c.bench_function("write", |b| {
        b.iter(|| write_path(black_box(&ferris), &WriteOptions::default()))
    });
    c.bench_function("optimize", |b| {
        b.iter(|| optimize(black_box(&ferris), Some(1)))
    });
    c.bench_function("transform", |b| {
        b.iter(|| {
            let mut path = PathTransformer::new(black_box(FERRIS).into());
            path.rotate(30.0, 600.0, 400.0)
                .scale(0.5, 0.5)
                .translate(10.0, 20.0);
            path.to_string()
        })
    });
}

fn measuring(c: &mut Criterion) {
    let ferris = parse(FERRIS);
    let measure = PathMeasure::new(&ferris);
    let total = measure.total_length();
    c.bench_function("measure", |b| {
        b.iter(|| PathMeasure::new(black_box(&ferris)))
    });
    c.bench_function("point_at x100", |b| {
        b.iter(|| {
            (0..100)
                .filter_map(|k| measure.point_at(total * f64::from(k) / 100.0))
                .map(|p| p.x)
                .sum::<f64>()
        })
    });
    c.bench_function("nearest", |b| {
        b.iter(|| measure.nearest(black_box(Point2D::new(600.0, 300.0))))
    });
    c.bench_function("contains", |b| {
        b.iter(|| measure.contains(black_box(Point2D::new(600.0, 300.0)), FillRule::NonZero))
    });
    c.bench_function("area", |b| b.iter(|| black_box(&measure).area()));
    c.bench_function("flatten 0.1", |b| {
        b.iter(|| measure.flatten(black_box(0.1)))
    });
    c.bench_function("stroke_bounds", |b| {
        b.iter(|| measure.stroke_bounds(black_box(&StrokeStyle::default())))
    });
}

fn geometry(c: &mut Criterion) {
    let ferris = parse(FERRIS);
    let circle = parse(CIRCLE);
    let measure = PathMeasure::new(&ferris);
    let circle_measure = PathMeasure::new(&circle);
    let lines = measure.flatten(0.1);
    let options = BooleanOptions { tolerance: 0.1, ..BooleanOptions::default() };
    c.bench_function("intersections", |b| {
        b.iter(|| measure.intersections(black_box(&circle_measure)))
    });
    c.bench_function("self_intersections", |b| {
        b.iter(|| black_box(&measure).self_intersections())
    });
    c.bench_function("simplify flattened", |b| {
        let lines = PathMeasure::new(&lines);
        b.iter(|| lines.simplify(black_box(0.5), 60.0))
    });
    c.bench_function("smooth", |b| {
        b.iter(|| measure.smooth(black_box(svg_path_ops::Smoothing::Continuous), 60.0))
    });
    c.bench_function("boolean difference", |b| {
        b.iter(|| boolean(black_box(&ferris), &circle, BooleanOp::Difference, &options))
    });
    c.bench_function("stroke_to_path", |b| {
        let style = StrokeStyle { width: 12.0, ..StrokeStyle::default() };
        b.iter(|| measure.stroke_to_path(black_box(&style), &options))
    });
    c.bench_function("offset", |b| {
        b.iter(|| measure.offset(black_box(8.0), LineJoin::Round, 4.0, &options))
    });
    c.bench_function("morph new", |b| {
        b.iter(|| Morph::new(black_box(&ferris), &circle))
    });
    let morph = Morph::new(&ferris, &circle);
    c.bench_function("morph at", |b| b.iter(|| morph.at(black_box(0.5))));
}

criterion_group!(benches, text, measuring, geometry);
criterion_main!(benches);
