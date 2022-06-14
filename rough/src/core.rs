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
    #[builder(setter(into, strip_option))]
    pub max_randomness_offset: Option<f32>,
    #[builder(setter(into, strip_option))]
    pub roughness: Option<f32>,
    #[builder(setter(into, strip_option))]
    pub bowing: Option<f32>,
    #[builder(setter(into, strip_option))]
    pub stroke: Option<String>,
    #[builder(setter(into, strip_option))]
    pub stroke_width: Option<f32>,
    #[builder(setter(into, strip_option))]
    pub curve_fitting: Option<f32>,
    #[builder(setter(into, strip_option))]
    pub curve_tightness: Option<f32>,
    #[builder(setter(into, strip_option))]
    pub curve_step_count: Option<f32>,
    #[builder(setter(into, strip_option))]
    pub fill: Option<String>,
    #[builder(setter(into, strip_option))]
    pub fill_style: Option<String>,
    #[builder(setter(into, strip_option))]
    pub fill_weight: Option<f32>,
    #[builder(setter(into, strip_option))]
    pub hachure_angle: Option<f32>,
    #[builder(setter(into, strip_option))]
    pub hachure_gap: Option<f32>,
    #[builder(setter(into, strip_option))]
    pub simplification: Option<f32>,
    #[builder(setter(into, strip_option))]
    pub dash_offset: Option<f32>,
    #[builder(setter(into, strip_option))]
    pub dash_gap: Option<f32>,
    #[builder(setter(into, strip_option))]
    pub zigzag_offset: Option<f32>,
    #[builder(setter(into, strip_option))]
    pub seed: Option<u64>,
    #[builder(setter(into, strip_option))]
    pub stroke_line_dash: Option<Vec<f32>>,
    #[builder(setter(into, strip_option))]
    pub stroke_line_dash_offset: Option<f32>,
    #[builder(setter(into, strip_option))]
    pub fill_line_dash: Option<Vec<f32>>,
    #[builder(setter(into, strip_option))]
    pub fill_line_dash_offset: Option<f32>,
    #[builder(setter(into, strip_option))]
    pub disable_multi_stroke: Option<bool>,
    #[builder(setter(into, strip_option))]
    pub disable_multi_stroke_fill: Option<bool>,
    #[builder(setter(into, strip_option))]
    pub preserve_vertices: Option<bool>,
    #[builder(setter(into, strip_option))]
    pub fixed_decimal_place_digits: Option<f32>,
    #[builder(setter(into, strip_option))]
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
