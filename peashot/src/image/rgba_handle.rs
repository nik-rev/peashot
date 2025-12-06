//! Wrapper around `iced::widget::image::Handle` to guarantee that it is an RGBA handle

use iced::{Rectangle, advanced::image::Bytes, widget::image::Handle};

/// The `RgbaHandle` is a wrapper for a handle pointing to decoded image pixels in RGBA format.
///
/// This is a more specialized version of `iced::widget::image::Handle`
#[derive(Debug, Clone)]
pub struct RgbaHandle(Handle);

impl RgbaHandle {
    /// Create handle to an image represented in RGBA format
    pub fn new(width: u32, height: u32, pixels: impl Into<Bytes>) -> Self {
        Self(Handle::from_rgba(width, height, pixels.into()))
    }

    /// Get the bounds of this image
    pub fn bounds(&self) -> Rectangle {
        Rectangle {
            x: 0.0,
            y: 0.0,
            width: self.width() as f32,
            height: self.height() as f32,
        }
    }

    /// Width of the image
    pub fn width(&self) -> u32 {
        self.raw().0
    }

    /// Height of the image
    pub fn height(&self) -> u32 {
        self.raw().1
    }

    /// RGBA bytes of the image
    pub fn bytes(&self) -> &Bytes {
        self.raw().2
    }

    /// Returns the width, height and RGBA pixels
    fn raw(&self) -> (u32, u32, &Bytes) {
        let Handle::Rgba {
            width,
            height,
            ref pixels,
            ..
        } = self.0
        else {
            unreachable!("handle is guaranteed to be Rgba")
        };
        (width, height, pixels)
    }
}

impl From<RgbaHandle> for Handle {
    fn from(value: RgbaHandle) -> Self {
        value.0
    }
}
