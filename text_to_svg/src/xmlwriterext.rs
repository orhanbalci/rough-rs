use ttf_parser::{self as ttf};

pub trait XmlWriterExt {
    fn write_color_attribute(&mut self, name: &str, ts: ttf::RgbaColor);
    fn write_transform_attribute(&mut self, name: &str, ts: ttf::Transform);
    fn write_spread_method_attribute(&mut self, method: ttf::colr::GradientExtend);
}

impl XmlWriterExt for xmlwriter::XmlWriter {
    fn write_color_attribute(&mut self, name: &str, color: ttf::RgbaColor) {
        self.write_attribute_fmt(
            name,
            format_args!("rgb({}, {}, {})", color.red, color.green, color.blue),
        );
    }

    fn write_transform_attribute(&mut self, name: &str, ts: ttf::Transform) {
        if ts.is_default() {
            return;
        }

        self.write_attribute_fmt(
            name,
            format_args!(
                "matrix({} {} {} {} {} {})",
                ts.a, ts.b, ts.c, ts.d, ts.e, ts.f
            ),
        );
    }

    fn write_spread_method_attribute(&mut self, extend: ttf::colr::GradientExtend) {
        self.write_attribute(
            "spreadMethod",
            match extend {
                ttf::colr::GradientExtend::Pad => &"pad",
                ttf::colr::GradientExtend::Repeat => &"repeat",
                ttf::colr::GradientExtend::Reflect => &"reflect",
            },
        );
    }
}
