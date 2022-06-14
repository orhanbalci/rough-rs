use euclid::{default::Point2D, Trig};
use num_traits::{Float, FromPrimitive};
use rand::{random, rngs::StdRng, RngCore, SeedableRng};

pub struct Space;

pub struct Config {
    options: Option<Options>,
}

pub struct DrawingSurface {
    width: f32,
    height: f32,
}

#[derive(Default, Clone, Builder)]
pub struct Options {
    pub max_randomness_offset: Option<f32>,
    pub roughness: Option<f32>,
    pub bowing: Option<f32>,
    pub stroke: Option<String>,
    pub stroke_width: Option<f32>,
    pub curve_fitting: Option<f32>,
    pub curve_tightness: Option<f32>,
    pub curve_step_count: Option<f32>,
    pub fill: Option<String>,
    pub fill_style: Option<String>,
    pub fill_weight: Option<f32>,
    pub hachure_angle: Option<f32>,
    pub hachure_gap: Option<f32>,
    pub simplification: Option<f32>,
    pub dash_offset: Option<f32>,
    pub dash_gap: Option<f32>,
    pub zigzag_offset: Option<f32>,
    pub seed: Option<u64>,
    pub stroke_line_dash: Option<Vec<f32>>,
    pub stroke_line_dash_offset: Option<f32>,
    pub fill_line_dash: Option<Vec<f32>>,
    pub fill_line_dash_offset: Option<f32>,
    pub disable_multi_stroke: Option<bool>,
    pub disable_multi_stroke_fill: Option<bool>,
    pub preserve_vertices: Option<bool>,
    pub fixed_decimal_place_digits: Option<f32>,
    pub randomizer: Option<StdRng>,
}

impl Options {
    pub fn random(&mut self) -> u64 {
        match &mut self.randomizer {
            Some(r) => r.next_u64(),
            None => match self.seed {
                Some(s) => {
                    let rnd = self.randomizer.insert(StdRng::seed_from_u64(s));
                    rnd.next_u64()
                }
                None => {
                    let rnd = self.randomizer.insert(StdRng::seed_from_u64(random()));
                    rnd.next_u64()
                }
            },
        }
    }
}

pub enum OpType {
    Move,
    BCurveTo,
    LineTo,
}

pub enum OpSetType {
    Path,
    FillPath,
    FillSketch,
}

pub struct Op<F: Float + Trig> {
    pub op: OpType,
    pub data: Vec<F>,
}

pub struct OpSet<F: Float + Trig> {
    pub op_set_type: OpSetType,
    pub ops: Vec<Op<F>>,
    pub size: Option<Point2D<F>>,
    pub path: Option<String>,
}

pub struct Drawable<F: Float + Trig> {
    pub shape: String,
    pub options: Options,
    pub sets: Vec<OpSet<F>>,
}

pub struct PathInfo {
    d: String,
    stroke: String,
    stroke_width: i32,
    fill: Option<String>,
}

pub fn _c<U: Float + FromPrimitive>(inp: f32) -> U {
    U::from(inp).expect("can not parse from f32")
}
