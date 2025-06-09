use iced::widget::canvas::{self, Frame, Geometry};
use iced::widget::{button, column, row, Canvas};
use iced::{Element, Length, Theme};
use iced::{Point, Rectangle};
use iced_widget::{pick_list, slider, text};
use num_traits::FloatConst;
use palette::Srgba;
use rough_iced::IcedGenerator;
use roughr::core::{FillStyle, OptionsBuilder};

#[derive(Debug, Clone, Copy)]
enum Shape {
    Arc,
    Rectangle,
    Circle,
}

#[derive(Debug, Clone)]
enum Message {
    ShapeSelected(Shape),
    FillStyleSelected(FillStyle),
    BowingChanged(f32),
    RoughnessChanged(f32),
    StrokeWidthChanged(f32),
}

struct DrawingApp {
    selected_shape: Shape,
    selected_fill_style: FillStyle,
    cache: canvas::Cache,
    bowing: f32,
    roughness: f32,
    stroke_width: f32,
}

impl Default for DrawingApp {
    fn default() -> Self {
        DrawingApp {
            selected_shape: Shape::Arc,
            cache: canvas::Cache::default(),
            selected_fill_style: FillStyle::Hachure,
            bowing: 2.0, // Default bowing value
            roughness: 1.0,
            stroke_width: 1.0, // Default stroke width
        }
    }
}

impl DrawingApp {
    fn title(&self) -> String {
        String::from("Drawing App")
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
        }
    }

    fn view(&self) -> Element<Message> {
        // Left panel with buttons for shape selection
        let shape_controls = column![
            button("Arc")
                .on_press(Message::ShapeSelected(Shape::Arc))
                .padding(10),
            button("Rectangle")
                .on_press(Message::ShapeSelected(Shape::Rectangle))
                .padding(10),
            button("Circle")
                .on_press(Message::ShapeSelected(Shape::Circle))
                .padding(10),
        ]
        .spacing(10)
        .padding(10);

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

        // Combine shape controls and fill style controls
        let controls = column![
            shape_controls,
            fill_style_controls,
            bowing_controls,
            roughness_controls,
            stroke_width_controls
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
            .fill_weight(1.0) // Adjust fill weight as needed
            .bowing(self.bowing)
            .roughness(self.roughness)
            .stroke_width(self.stroke_width)
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
        }

        vec![frame.into_geometry()]
    }
}
pub fn main() -> iced::Result {
    iced::application(DrawingApp::title, DrawingApp::update, DrawingApp::view)
        .theme(|_| Theme::CatppuccinMocha)
        .run()
}
