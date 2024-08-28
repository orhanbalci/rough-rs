#![feature(iter_map_windows)]

use std::path::Path;

use options::Options;
use ttf_parser::{self as ttf, Face};
mod options;

pub struct TextToSvg<'a> {
    pub font: Face<'a>,
}

impl<'a> TextToSvg<'a> {
    pub fn new<P>(font_data: &'a Vec<u8>) -> Self
    where
        P: AsRef<Path>,
    {
        let face: Face<'a> = ttf::Face::parse(&font_data, 0).expect("unable to parse font file");
        TextToSvg { font: face }
    }

    pub fn get_width(&self, text: String, options: Options) -> f32 {
        let font_size = options.fontsize;
        let kerning = options.kerning;
        let font_scale: f32 = font_size as f32 / self.font.units_per_em() as f32;

        let mut width: f32 = 0.0;

        width += text
            .chars()
            .map(|c| {
                let index = self.font.glyph_index(c);
                return index
                    .map(|i| self.font.glyph_hor_advance(i).unwrap_or_default())
                    .unwrap_or_default() as f32
                    * font_scale;
            })
            .sum::<f32>();

        if kerning {
            if let Some(kerning_table) = self.font.tables().kern {
                width += text
                    .chars()
                    .map_windows(|[char1, char2]| {
                        let index1 = self.font.glyph_index(*char1);
                        let index2 = self.font.glyph_index(*char2);
                        if index1.and(index2).is_some() {
                            if let Some(kern_subtable) = kerning_table.subtables.into_iter().next()
                            {
                                return kern_subtable
                                    .glyphs_kerning(index1.unwrap(), index2.unwrap())
                                    .unwrap_or_default()
                                    as f32;
                            }
                        }
                        return 0.0;
                    })
                    .sum::<f32>();
            }
        }

        if let Some(ls) = options.letter_spacing {
            width += text.chars().count() as f32 * (ls * font_size) as f32;
        } else if let Some(tr) = options.tracking {
            width += text.chars().count() as f32 * (tr as f32 / 1000.0) * font_size as f32;
        }

        return width;
    }

    pub fn get_height(&self, font_size: u16) -> f32 {
        let font_scale = font_size as f32 / self.font.units_per_em() as f32;
        return (self.font.ascender() - self.font.descender()) as f32 * font_scale;
    }
}
