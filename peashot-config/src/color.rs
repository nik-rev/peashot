use core::fmt;

use facet::Facet;
use facet_kdl as kdl;

/// A name that a color is allowed to have
#[derive(Facet, Debug, PartialEq, Hash, Eq)]
#[facet(transparent)]
pub struct ColorId(String);

#[derive(Facet, Debug, PartialEq)]
pub struct KdlColor {
    #[facet(kdl::argument, proxy = KdlColorValueProxy)]
    value: KdlColorValue,
    // #[facet(kdl::property, default)]
    // opacity: Option<f32>,
}

crate::create_string_proxy!(KdlColorValue <=> KdlColorValueProxy);

#[derive(Facet, Debug, PartialEq)]
#[repr(C)]
pub enum KdlColorValue {
    Color(Color),
    Palette(PaletteId),
}

impl std::str::FromStr for KdlColorValue {
    type Err = csscolorparser::ParseColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(s) = s.strip_prefix('@') {
            // Referencing a color in the palette, which we'll resolve later
            Ok(Self::Palette(PaletteId(s.to_string())))
        } else {
            csscolorparser::parse(s).map(Into::into)
        }
    }
}

impl fmt::Display for KdlColorValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KdlColorValue::Color(color) => f.write_fmt(format_args!(
                "#{:02}{:02}{:02}{:02}",
                color.r, color.g, color.b, color.a
            )),
            KdlColorValue::Palette(palette_id) => f.write_str(&palette_id.0.to_string()),
        }
    }
}

#[derive(Facet, Debug, PartialEq, Hash, Eq)]
#[facet(transparent)]
pub struct PaletteId(String);

#[derive(Debug, PartialEq, Facet)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl From<csscolorparser::Color> for Color {
    fn from(color: csscolorparser::Color) -> Self {
        let csscolorparser::Color { r, g, b, a } = color;
        Self { r, g, b, a }
    }
}

impl From<csscolorparser::Color> for KdlColorValue {
    fn from(color: csscolorparser::Color) -> Self {
        KdlColorValue::Color(color.into())
    }
}
