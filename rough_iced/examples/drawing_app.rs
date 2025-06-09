use euclid::Point2D;
use iced::widget::canvas::{self, Frame, Geometry};
use iced::widget::{button, column, row, Canvas};
use iced::{Element, Length, Theme};
use iced::{Point, Rectangle};
use iced_widget::{pick_list, slider, text};
use num_traits::FloatConst;
use palette::Srgba;
use rough_iced::IcedGenerator;
use roughr::core::{FillStyle, OptionsBuilder};
use svg_path_ops::pt::PathTransformer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shape {
    Arc,
    Rectangle,
    Circle,
    Ellipse,
    BezierCubic, // New Bezier Cubic shape
    Heart,
}

impl ToString for Shape {
    fn to_string(&self) -> String {
        match self {
            Shape::Arc => "Arc".to_string(),
            Shape::Rectangle => "Rectangle".to_string(),
            Shape::Circle => "Circle".to_string(),
            Shape::Ellipse => "Ellipse".to_string(),
            Shape::BezierCubic => "Bezier Cubic".to_string(),
            Shape::Heart => "Heart".to_string(), // Display name for Bezier Cubic
        }
    }
}

const SHAPES: [Shape; 6] = [
    Shape::Arc,
    Shape::Rectangle,
    Shape::Circle,
    Shape::Ellipse,
    Shape::BezierCubic,
    Shape::Heart,
];

#[derive(Debug, Clone)]
enum Message {
    ShapeSelected(Shape),
    FillStyleSelected(FillStyle),
    BowingChanged(f32),
    RoughnessChanged(f32),
    StrokeWidthChanged(f32),
    CurveFittingChanged(f32),
    CurveTightnessChanged(f32),
    CurveStepCountChanged(f32),
    FillWeightChanged(f32),
    HachureAngleChanged(f32),
    HachureGapChanged(f32),
    SimplificationChanged(f32),
    DashOffsetChanged(f32),
    DashGapChanged(f32),
    ZigzagOffsetChanged(f32),
}

struct DrawingApp {
    selected_shape: Shape,
    selected_fill_style: FillStyle,
    cache: canvas::Cache,
    bowing: f32,
    roughness: f32,
    stroke_width: f32,
    curve_fitting: f32,
    curve_tightness: f32,
    curve_step_count: f32,
    fill_weight: f32,
    hachure_angle: f32,
    hachure_gap: f32,
    simplification: f32,
    dash_offset: f32,
    dash_gap: f32,
    zigzag_offset: f32,
}

impl Default for DrawingApp {
    fn default() -> Self {
        DrawingApp {
            selected_shape: Shape::Rectangle,
            cache: canvas::Cache::default(),
            selected_fill_style: FillStyle::Hachure,
            bowing: 2.0, // Default bowing value
            roughness: 1.0,
            stroke_width: 1.0,   // Default stroke width
            curve_fitting: 0.95, // Default curve fitting value
            curve_tightness: 0.0,
            curve_step_count: 9.0, // Default curve step count value
            fill_weight: 1.0,
            hachure_angle: -41.0,
            hachure_gap: -1.0,
            simplification: 1.0,
            dash_offset: -1.0,
            dash_gap: -1.0,
            zigzag_offset: -1.0,
        }
    }
}

impl DrawingApp {
    fn title(&self) -> String {
        String::from("Rough Configuration App")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ShapeSelected(shape) => {
                self.selected_shape = shape;
                self.cache.clear(); // Clear the canvas cache to redraw
            }
            Message::FillStyleSelected(fill_style) => {
                self.selected_fill_style = fill_style;
                self.cache.clear(); // Clear the canvas cache to redraw
            }
            Message::BowingChanged(bowing) => {
                self.bowing = bowing;
                self.cache.clear(); // Clear the canvas cache to redraw
            }
            Message::RoughnessChanged(roughness) => {
                self.roughness = roughness;
                self.cache.clear(); // Clear the canvas cache to redraw
            }
            Message::StrokeWidthChanged(stroke_width) => {
                self.stroke_width = stroke_width;
                self.cache.clear(); // Clear the canvas cache to redraw
            }
            Message::CurveFittingChanged(curve_fitting) => {
                self.curve_fitting = curve_fitting;
                self.cache.clear(); // Clear the canvas cache to redraw
            }
            Message::CurveTightnessChanged(curve_tightness) => {
                self.curve_tightness = curve_tightness;
                self.cache.clear(); // Clear the canvas cache to redraw
            }
            Message::CurveStepCountChanged(curve_step_count) => {
                self.curve_step_count = curve_step_count;
                self.cache.clear(); // Clear the canvas cache to redraw
            }
            Message::FillWeightChanged(fill_weight) => {
                self.fill_weight = fill_weight;
                self.cache.clear(); // Clear the canvas cache to redraw
            }
            Message::HachureAngleChanged(hachure_angle) => {
                self.hachure_angle = hachure_angle;
                self.cache.clear(); // Clear the canvas cache to redraw
            }
            Message::HachureGapChanged(hachure_gap) => {
                self.hachure_gap = hachure_gap;
                self.cache.clear(); // Clear the canvas cache to redraw
            }
            Message::SimplificationChanged(simplification) => {
                self.simplification = simplification;
                self.cache.clear(); // Clear the canvas cache to redraw
            }
            Message::DashOffsetChanged(dash_offset) => {
                self.dash_offset = dash_offset;
                self.cache.clear(); // Clear the canvas cache to redraw
            }
            Message::DashGapChanged(dash_gap) => {
                self.dash_gap = dash_gap;
                self.cache.clear(); // Clear the canvas cache to redraw
            }
            Message::ZigzagOffsetChanged(zigzag_offset) => {
                self.zigzag_offset = zigzag_offset;
                self.cache.clear(); // Clear the canvas cache to redraw
            }
        }
    }

    fn view(&self) -> Element<Message> {
        // Left panel with buttons for shape selection
        // Shape selection dropdown
        let shape_controls =
            pick_list(SHAPES, Some(self.selected_shape), Message::ShapeSelected).padding(10);

        // Fill style selection dropdown
        let fill_styles = [
            FillStyle::Solid,
            FillStyle::Hachure,
            FillStyle::ZigZag,
            FillStyle::CrossHatch,
            FillStyle::Dots,
            FillStyle::Dashed,
            FillStyle::ZigZagLine,
        ];
        let fill_style_controls = pick_list(
            fill_styles,
            Some(self.selected_fill_style),
            Message::FillStyleSelected,
        )
        .padding(10);

        let bowing_controls = column![
            text(format!("Bowing: {:.1}", self.bowing)).size(16),
            slider(0.0..=20.0, self.bowing, Message::BowingChanged).step(0.1),
        ]
        .spacing(10);

        let roughness_controls = column![
            text(format!("Roughness: {:.1}", self.roughness)).size(16),
            slider(0.0..=20.0, self.roughness, Message::RoughnessChanged).step(0.1),
        ]
        .spacing(10);

        // Stroke width slider
        let stroke_width_controls = column![
            text(format!("Stroke Width: {:.1}", self.stroke_width)).size(16),
            slider(0.8..=10.0, self.stroke_width, Message::StrokeWidthChanged).step(0.1),
        ]
        .spacing(10);

        // Curve fitting slider
        let curve_fitting_controls = column![
            text(format!("Curve Fitting: {:.2}", self.curve_fitting)).size(16),
            slider(0.0..=1.0, self.curve_fitting, Message::CurveFittingChanged).step(0.01),
        ]
        .spacing(10);

        // Curve tightness slider
        let curve_tightness_controls = column![
            text(format!("Curve Tightness: {:.2}", self.curve_tightness)).size(16),
            slider(
                -10.0..=10.0,
                self.curve_tightness,
                Message::CurveTightnessChanged
            )
            .step(0.1),
        ]
        .spacing(10);

        // Curve step count slider
        let curve_step_count_controls = column![
            text(format!("Curve Step Count: {:.1}", self.curve_step_count)).size(16),
            slider(
                1.0..=20.0,
                self.curve_step_count,
                Message::CurveStepCountChanged
            )
            .step(1.0),
        ]
        .spacing(10);

        // Fill weight slider
        let fill_weight_controls = column![
            text(format!("Fill Weight: {:.1}", self.fill_weight)).size(16),
            slider(0.1..=10.0, self.fill_weight, Message::FillWeightChanged).step(0.1),
        ]
        .spacing(10);

        // Hachure angle slider
        let hachure_angle_controls = column![
            text(format!("Hachure Angle: {:.1}", self.hachure_angle)).size(16),
            slider(
                -90.0..=90.0,
                self.hachure_angle,
                Message::HachureAngleChanged
            )
            .step(1.0),
        ]
        .spacing(10);

        // Hachure gap slider
        let hachure_gap_controls = column![
            text(format!("Hachure Gap: {:.1}", self.hachure_gap)).size(16),
            slider(0.0..=20.0, self.hachure_gap, Message::HachureGapChanged).step(0.1),
        ]
        .spacing(10);

        // Simplification slider
        let simplification_controls = column![
            text(format!("Simplification: {:.2}", self.simplification)).size(16),
            slider(
                0.0..=1.0,
                self.simplification,
                Message::SimplificationChanged
            )
            .step(0.1),
        ]
        .spacing(10);

        // Dash offset slider
        let dash_offset_controls = column![
            text(format!("Dash Offset: {:.1}", self.dash_offset)).size(16),
            slider(-10.0..=10.0, self.dash_offset, Message::DashOffsetChanged).step(0.1),
        ]
        .spacing(10);

        // Dash gap slider
        let dash_gap_controls = column![
            text(format!("Dash Gap: {:.1}", self.dash_gap)).size(16),
            slider(0.0..=20.0, self.dash_gap, Message::DashGapChanged).step(0.1),
        ]
        .spacing(10);

        // Zigzag offset slider
        let zigzag_offset_controls = column![
            text(format!("Zigzag Offset: {:.1}", self.zigzag_offset)).size(16),
            slider(1.0..=10.0, self.zigzag_offset, Message::ZigzagOffsetChanged).step(1.01),
        ]
        .spacing(10);

        // Combine shape controls and fill style controls
        let controls = column![
            shape_controls,
            fill_style_controls,
            bowing_controls,
            roughness_controls,
            stroke_width_controls,
            curve_fitting_controls,
            curve_tightness_controls,
            curve_step_count_controls,
            fill_weight_controls,
            hachure_angle_controls,
            hachure_gap_controls,
            simplification_controls,
            dash_offset_controls,
            dash_gap_controls,
            zigzag_offset_controls
        ]
        .spacing(20);

        // Canvas for drawing
        let canvas = Canvas::new(self).width(Length::Fill).height(Length::Fill);

        // Combine the controls and canvas in a row
        row![controls, canvas]
            .spacing(10)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl<Message> canvas::Program<Message> for DrawingApp {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        // Define rough.js-style options
        let options = OptionsBuilder::default()
            .stroke(Srgba::from_components((114u8, 87u8, 82u8, 255u8)).into_format())
            .fill(Srgba::from_components((254u8, 246u8, 201u8, 255u8)).into_format())
            .fill_style(self.selected_fill_style)
            .bowing(self.bowing)
            .roughness(self.roughness)
            .stroke_width(self.stroke_width)
            .curve_fitting(self.curve_fitting)
            .curve_tightness(self.curve_tightness)
            .curve_step_count(self.curve_step_count)
            .fill_weight(self.fill_weight)
            .hachure_angle(self.hachure_angle)
            .hachure_gap(self.hachure_gap)
            .simplification(self.simplification)
            .dash_offset(self.dash_offset)
            .dash_gap(self.dash_gap)
            .zigzag_offset(self.zigzag_offset)
            .build()
            .unwrap();

        // Create a rough generator
        let generator = IcedGenerator::new(options);

        // Draw the background
        frame.fill_rectangle(
            Point::ORIGIN,
            iced::Size::new(bounds.width, bounds.height),
            iced::Color::from_rgb8(150, 192, 183), // Background color
        );
        // Draw based on the selected shape
        match self.selected_shape {
            Shape::Arc => {
                let arc_path = generator.arc(
                    bounds.width / 2.0,
                    bounds.height / 2.0,
                    bounds.width / 2.0,
                    bounds.height / 2.0,
                    -f32::PI() / 2.0,
                    f32::PI() / 2.0,
                    false,
                );
                arc_path.draw(&mut frame);
            }
            Shape::Rectangle => {
                let rect_path = generator.rectangle(
                    bounds.width / 4.0,
                    bounds.height / 4.0,
                    bounds.width / 2.0,
                    bounds.height / 2.0,
                );
                rect_path.draw(&mut frame);
            }
            Shape::Circle => {
                let circle_path =
                    generator.circle(bounds.width / 2.0, bounds.height / 2.0, bounds.width / 3.0);
                circle_path.draw(&mut frame);
            }
            Shape::Ellipse => {
                let ellipse_path = generator.ellipse(
                    (bounds.width as f32) / 2.0,
                    (bounds.height as f32) / 2.0,
                    bounds.width / 2.0,
                    bounds.height / 3.0,
                );
                ellipse_path.draw(&mut frame);
            }
            Shape::BezierCubic => {
                let bezier_path = generator.bezier_cubic(
                    Point2D::new(bounds.width / 4.0, bounds.height / 2.0),
                    Point2D::new(bounds.width / 3.0, bounds.height / 4.0),
                    Point2D::new(2.0 * bounds.width / 3.0, 3.0 * bounds.height / 4.0),
                    Point2D::new(3.0 * bounds.width / 4.0, bounds.height / 2.0),
                );
                bezier_path.draw(&mut frame);
            }
            Shape::Heart => {
                let heart_svg_path = "M140 20C73 20 20 74 20 140c0 135 136 170 228 303 88-132 229-173 229-303 0-66-54-120-120-120-48 0-90 28-109 69-19-41-60-69-108-69z".into();
                let mut translated_path = PathTransformer::new(heart_svg_path);
                let bbox = translated_path.to_box(Some(1));
                translated_path.translate(
                    bounds.width as f64 / 2.0 - bbox.width() / 2.0,
                    bounds.height as f64 / 2.0 - bbox.height() / 2.0,
                );

                let translated_path_string = translated_path.to_string();
                let heart_path = generator.path::<f32>(translated_path_string);
                heart_path.draw(&mut frame);
            }
        }

        vec![frame.into_geometry()]
    }
}
pub fn main() -> iced::Result {
    iced::application(DrawingApp::title, DrawingApp::update, DrawingApp::view)
        .theme(|_| Theme::CatppuccinMocha)
        .run()
}
