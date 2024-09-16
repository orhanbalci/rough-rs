use cgmath::{Angle, Deg, Matrix3, Vector2, Vector3};
use svgtypes::{PathParser, PathSegment, TransformListParser, TransformListToken};

pub struct PathTransformer {
    path_segments: Vec<PathSegment>,
    stack: Vec<Matrix3<f64>>,
}

impl PathTransformer {
    pub fn new(path: String) -> Self {
        let mut path_parser = PathParser::from(path.as_ref());
        if path_parser.any(|a| a.is_err()) {
            panic!("unexpected path string. can not parse it.")
        }

        PathTransformer {
            path_segments: path_parser
                .filter(|ps| ps.is_ok())
                .map(|ps| ps.unwrap())
                .collect(),
            stack: Vec::new(),
        }
    }

    pub fn translate(&mut self, tx: f64, ty: f64) -> &mut Self {
        self.stack
            .push(Matrix3::from_translation(Vector2::new(tx, ty)));
        self
    }

    pub fn scale(&mut self, sx: f64, sy: f64) -> &mut Self {
        self.stack.push(Matrix3::from_nonuniform_scale(sx, sy));
        self
    }

    pub fn rotate(&mut self, angle: f64, rx: f64, ry: f64) -> &mut Self {
        self.stack.push(Matrix3::from_axis_angle(
            Vector3::new(rx, ry, 0.0),
            Deg(angle),
        ));
        self
    }

    pub fn skew_x(&mut self, degrees: f64) -> &mut Self {
        let skew_xmatrix = Matrix3::new(1.0, 0.0, 0.0, Deg(degrees).tan(), 1.0, 0.0, 0.0, 0.0, 1.0);
        self.stack.push(skew_xmatrix);
        self
    }

    pub fn skew_y(&mut self, degrees: f64) -> &mut Self {
        let skew_ymatrix = Matrix3::new(1.0, Deg(degrees).tan(), 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0);
        self.stack.push(skew_ymatrix);
        self
    }

    pub fn matrix(&mut self, matrix: [f64; 6]) -> &mut Self {
        let converted = Matrix3::new(
            matrix[0], matrix[3], 0.0, matrix[1], matrix[4], 0.0, matrix[2], matrix[5], 1.0,
        );
        self.stack.push(converted);
        self
    }

    pub fn transform(&mut self, transform: String) -> &mut Self {
        let parser = TransformListParser::from(transform.as_str());
        for path_transform in parser {
            match path_transform {
                Ok(pt) => {
                    self.apply_token(pt);
                }
                Err(_) => {
                    println!("Can not parse transform string.");
                }
            }
        }
        self
    }

    fn apply_token(&mut self, token: TransformListToken) -> &mut Self {
        match token {
            TransformListToken::Matrix { a, b, c, d, e, f } => self.matrix([a, b, c, d, e, f]),
            TransformListToken::Translate { tx, ty } => self.translate(tx, ty),
            TransformListToken::Scale { sx, sy } => self.scale(sx, sy),
            TransformListToken::Rotate { angle } => self.rotate(angle, 0.0, 0.0),
            TransformListToken::SkewX { angle } => self.skew_x(angle),
            TransformListToken::SkewY { angle } => self.skew_y(angle),
        };

        self
    }
}
