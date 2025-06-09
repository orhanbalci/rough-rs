//! This example demonstrates how to draw a rough arc using the `rough_iced` crate.

use iced::mouse::Cursor;
use iced::widget::canvas::{self, Canvas, Frame, Geometry};
use iced::{Application, Command, Element, Length, Point, Rectangle, Renderer, Settings, Theme};
use num_traits::FloatConst;
use palette::Srgba;
use rough_iced::IcedGenerator;
use roughr::core::{FillStyle, OptionsBuilder};

/// Main application struct
struct ArcExample;

impl Application for ArcExample {
    type Executor = iced::executor::Default;
    type Message = ();
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self, Command::none())
    }

    fn title(&self) -> String {
        String::from("Rough Arc Example")
    }

    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        Canvas::new(ArcCanvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

/// Custom canvas for drawing the arc
struct ArcCanvas;

impl canvas::Program<()> for ArcCanvas {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        // Define rough.js-style options
        let options = OptionsBuilder::default()
            .stroke(Srgba::from_components((114u8, 87u8, 82u8, 255u8)).into_format())
            .fill(Srgba::from_components((254u8, 246u8, 201u8, 255u8)).into_format())
            .fill_style(FillStyle::Hachure)
            .fill_weight(1.0) // Adjust fill weight as needed
            .build()
            .unwrap();

        // Create a rough generator
        let generator = IcedGenerator::new(options);

        // Generate the arc path
        let arc_path = generator.arc(
            bounds.width / 2.0,
            bounds.height / 2.0,
            bounds.width / 2.0,
            bounds.height / 2.0,
            -f32::PI() / 2.0,
            f32::PI() / 2.0,
            false,
        );

        // Draw the background
        frame.fill_rectangle(
            Point::ORIGIN,
            iced::Size::new(bounds.width, bounds.height),
            iced::Color::from_rgb8(150, 192, 183), // Background color
        );

        // Draw the arc
        arc_path.draw(&mut frame);

        vec![frame.into_geometry()]
    }
}

fn main() {
    ArcExample::run(Settings { antialiasing: true, ..Settings::default() });
}
