//! Parse "lazy rect" using custom syntax to specify precise dimensions
//! It is "lazy" because the concrete size of it will depend on the size of its container
//!
//! Syntax:
//!
//! WxH+X+Y[-XP%[+YP%]]
//!
//! Where:
//! - W: width of the rectangle
//! - H: height of the rectangle
//! - X: x-coordinate of the top-left corner of the rectangle
//! - Y: y-coordinate of the top-left corner of the rectangle
//! - XP: percentage of `W` to add to `X`
//! - YP: percentage of `H` to add to `Y`
//!
//! With `XP` and `YP` it allows us to have more control over how the rectangle
//! is positioned. For instance, we can create a 100x150 rectangle that is centered with:
//!
//! - 100x150+0.5+0.5-50%-50%
//!
//! Explanation:
//! - Width is 100
//! - Height is 150
//! - X is in the center of width of rectangle
//! - Y is in the center of height of rectangle
//!
//! Now, the above would mean the top-left corner of the rectangle
//! is in the center of the image.
//!
//! - -50% moves it to the left by 50px, -50% * 100px (width) = -50px
//! - similar with height, but -50% * 150px (width) = -75px

use std::{
    fmt,
    num::{ParseFloatError, ParseIntError},
    str::FromStr,
};

use iced::Rectangle;

use crate::geometry::RectangleExt as _;

/// Percentage
#[derive(Clone, Copy, Debug, PartialEq)]
struct Percentage(f32);

/// Error parsing percentage like "0.47"
#[derive(thiserror::Error, miette::Diagnostic, Debug, Clone, Eq, PartialEq)]
pub enum ParsePercentageError {
    /// Parse float error
    #[error(transparent)]
    ParseFloatError(ParseFloatError),
}

impl fmt::Display for Percentage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Percentage {
    type Err = ParsePercentageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<f32>() {
            Ok(n) => Ok(Self(n)),
            Err(err) => Err(ParsePercentageError::ParseFloatError(err)),
        }
    }
}

/// Represents a single axis / dimension of the rectangle's x, y, width, or height values
#[derive(Clone, Copy, Debug, PartialEq)]
enum Length {
    /// A specific amount of pixels
    Absolute(u32),
    /// Percentage of another value
    Relative(Percentage),
}

impl fmt::Display for Length {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&match self {
            Self::Absolute(n) => n.to_string(),
            Self::Relative(n) => n.to_string(),
        })
    }
}

impl Length {
    /// Convert this into an `f32`
    pub fn into_f32(self, other: f32) -> f32 {
        match self {
            Self::Absolute(n) => n as f32,
            Self::Relative(n) => n.0 * other,
        }
    }
}

/// Error parsing a num
#[derive(thiserror::Error, miette::Diagnostic, Debug, Clone, Eq, PartialEq)]
pub enum ParseLengthError {
    /// Parse percentage error
    #[error(transparent)]
    ParsePercentageError(ParsePercentageError),
    /// Parse int error
    #[error(transparent)]
    ParseIntError(ParseIntError),
}

impl FromStr for Length {
    type Err = ParseLengthError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u32>().map_or_else(
            |_| match s.parse::<Percentage>() {
                Ok(percent) => Ok(Self::Relative(percent)),
                Err(percent_err) => Err(ParseLengthError::ParsePercentageError(percent_err)),
            },
            |n| Ok(Self::Absolute(n)),
        )
    }
}

/// Nudge a coordinate by an amount
#[derive(Debug, Clone, Copy, PartialEq)]
struct Nudge {
    /// Move the `original_position` by this amount, relative to the width / height of the rect
    /// It is a `Percentage` because if e.g. `width` is a `Percentage`, we don't know the concrete
    /// amount and we'll calculate that later
    by: Percentage,
    /// Whether we should nudge it
    /// - left or up
    /// - right or down
    is_negative: bool,
}

/// Represents a coordinate
///
/// The coordinate can be "nudged", that is, say we provide a
/// -40% value. Now the `x` coordinate will move to the left
/// by 0.4 * Width of the rectangle
#[derive(Debug, Clone, Copy, PartialEq)]
struct Coord {
    /// Original coordinate position, before it is nudged
    original_position: Length,
    /// Nudge the original position by some amount
    nudge: Option<Nudge>,
}

/// The rectangle will turn into an `iced::Rectangle` once we know the bounds of its container
///
/// The container will be the image that we are editing
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LazyRectangle {
    /// x of the top left corner
    x: Coord,
    /// y of the top-left corner
    y: Coord,
    /// width
    width: Length,
    /// height
    height: Length,
}

impl LazyRectangle {
    /// Expands to its entire container
    pub const FULL: Self = Self {
        x: Coord {
            original_position: Length::Absolute(0),
            nudge: None,
        },
        y: Coord {
            original_position: Length::Absolute(0),
            nudge: None,
        },
        width: Length::Relative(Percentage(1.0)),
        height: Length::Relative(Percentage(1.0)),
    };

    /// Convert this type into an `iced::Rectangle`,
    /// with knowing the `bounds` that it will be inside
    ///
    /// The produced `Rectangle` is guaranteed not to exceed the `bounds`
    pub fn init(self, bounds: Rectangle) -> Rectangle {
        let x = self.x.original_position.into_f32(bounds.width)
            + self.x.nudge.map_or(0.0, |nudge| {
                let sign = if nudge.is_negative { -1 } else { 1 };

                let nudge_by_px = match self.width {
                    Length::Absolute(n) => n as f32 * nudge.by.0,
                    Length::Relative(n) => n.0 * nudge.by.0 * bounds.width,
                };

                sign as f32 * nudge_by_px
            });

        let y = self.y.original_position.into_f32(bounds.height)
            + self.y.nudge.map_or(0.0, |nudge| {
                let sign = if nudge.is_negative { -1 } else { 1 };

                let nudge_by_px = match self.height {
                    Length::Absolute(n) => n as f32 * nudge.by.0,
                    Length::Relative(n) => n.0 * nudge.by.0 * bounds.height,
                };

                sign as f32 * nudge_by_px
            });

        let width = self.width.into_f32(bounds.width);
        let height = self.height.into_f32(bounds.height);

        Rectangle {
            x,
            y,
            width,
            height,
        }
        .clipped_in_bounds_of(bounds)
    }
}

/// Error parsing a rect
#[derive(thiserror::Error, miette::Diagnostic, Debug, Clone, Eq, PartialEq)]
#[error("Failed to parse region")]
#[diagnostic(help(
    "use the valid format: `<width>x<height>+<top-left-x>+<top-left-y>`, like 100x90+75+80"
))]
pub enum ParseRectError {
    /// Missing % sign
    #[error("Missing % sign")]
    MissingPercentage,
    /// Missing positive / negative sign
    #[error("Invalid sign, expected `-` or `+` but found {0}")]
    InvalidSign(String),
    /// Lazy rectangle is malformed
    #[error("Invalid format")]
    InvalidFormat,
    /// Failed to parse the percentage.
    #[error(transparent)]
    PercentageParseError(#[from] ParsePercentageError),
    /// Failed to parse a float.
    #[error(transparent)]
    ParseLengthError(#[from] ParseLengthError),
}

impl FromStr for LazyRectangle {
    type Err = ParseRectError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "full" {
            // the inner rect will be the same size as the outer rect
            return Ok(Self::FULL);
        }

        // Example of a `LazyRectangle` string being parsed.
        // Quotes are omitted
        //
        // START: 100x250+180+190-50%+75%
        let (width, height, x, left) = s
            .split_once('x')
            // (100), (250+180+190-50%+75%)
            .and_then(|(width, rest)| {
                rest.split_once('+')
                    // (100), ((250), (180+190-50%+75%))
                    .and_then(|(height, rest)| {
                        rest.split_once('+').map(
                            |(x, y)| (height, x, y), // (100, (250, 180, 190-50%+75%))
                        )
                    })
                    .map(|(height, x, y)| (width, height, x, y))
                // (100, 250, 180, 190-50%+75%)
            })
            .ok_or(ParseRectError::InvalidFormat)?;

        // Now we will break up the 190-50%+75% into:
        // ("190", Some("-50"), Some("75"))

        let (y, x_nudge, y_nudge) = {
            let bytes = left.as_bytes();

            let mut idx = 0;
            while idx < bytes.len() && (bytes[idx].is_ascii_digit() || bytes[idx] == b'.') {
                idx += 1;
            }

            // y = "190"
            //
            // &left[idx..] = "-50%+75%"
            let y = &left[..idx];

            // y is the entire length, meaning no nudge for either X or Y
            if idx == left.len() {
                (y, None, None)
            } else {
                let both_nudges = &left[idx..];
                let mut nudges = Vec::new();
                let mut start = 0;

                // Find modifier parts, tracking the sign
                while start < both_nudges.len() {
                    let sign = &both_nudges[start..=start];
                    let next = both_nudges[start + 1..]
                        .find(['+', '-'])
                        .map_or(both_nudges.len(), |i| i + start + 1);

                    let mut part = &both_nudges[start + 1..next];
                    if part.ends_with('%') {
                        part = &part[..part.len() - 1];
                    } else {
                        return Err(ParseRectError::MissingPercentage);
                    }

                    if sign == "-" {
                        nudges.push(Some((part, true)));
                    } else {
                        nudges.push(Some((part, false)));
                    }

                    start = next;
                }

                let parse_nudge = |i| {
                    if nudges.len() > 2 {
                        return Err(ParseRectError::InvalidFormat);
                    }

                    nudges
                        .get(i)
                        .copied()
                        .flatten()
                        .map(|(num, is_negative): (&str, bool)| {
                            if num == "100" {
                                "1.0".to_string()
                            } else {
                                format!("0.{num}")
                            }
                            .parse::<Percentage>()
                            .map_err(ParseRectError::PercentageParseError)
                            .map(|by| Nudge { by, is_negative })
                        })
                        .transpose()
                };

                // x_nudge = "-50%"
                let x_nudge = parse_nudge(0)?;
                // y_nudge = "+75%"
                let y_nudge = parse_nudge(1)?;

                (y, x_nudge, y_nudge)
            }
        };

        Ok(Self {
            x: Coord {
                original_position: x.parse()?,
                nudge: x_nudge,
            },
            y: Coord {
                original_position: y.parse()?,
                nudge: y_nudge,
            },
            width: width.parse()?,
            height: height.parse()?,
        })
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, reason = "small values")]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    use iced::Rectangle;

    #[test]
    fn basic_absolute() {
        assert_eq!(
            "100x200+10+20"
                .parse::<LazyRectangle>()
                .unwrap()
                .init(Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: 1920.0,
                    height: 1080.0,
                }),
            Rectangle {
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 200.0
            }
        );
    }

    /// Rectangle gets clapped into bounds
    #[test]
    fn clamp_exceeding_bounds() {
        assert_eq!(
            "500x500+0+0"
                .parse::<LazyRectangle>()
                .unwrap()
                .init(Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: 300.0,
                    height: 200.0,
                }),
            Rectangle {
                x: 0.0,
                y: 0.0,
                width: 300.0,
                height: 200.0
            }
        );
    }

    #[test]
    fn full_keyword() {
        let bounds = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 1234.0,
            height: 567.0,
        };
        assert_eq!(
            "full".parse::<LazyRectangle>().unwrap().init(bounds),
            bounds
        );
    }

    #[test]
    fn invalid_format() {
        assert_eq!(
            "abc".parse::<LazyRectangle>().unwrap_err(),
            ParseRectError::InvalidFormat
        );
    }

    #[test]
    fn zero_size() {
        assert_eq!(
            "0x0+10+10"
                .parse::<LazyRectangle>()
                .unwrap()
                .init(Rectangle {
                    width: 500.0,
                    height: 500.0,
                    x: 0.0,
                    y: 0.0,
                }),
            Rectangle {
                width: 0.0,
                height: 0.0,
                x: 10.0,
                y: 10.0,
            }
        );
    }

    #[test]
    fn percent_position_and_size() {
        assert_eq!(
            "0.5x0.5+0.25+0.25"
                .parse::<LazyRectangle>()
                .unwrap()
                .init(Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: 1000.0,
                    height: 800.0,
                }),
            Rectangle {
                width: 500.0,
                height: 400.0,
                x: 250.0,
                y: 200.0,
            }
        );
    }

    #[test]
    fn nudged_negative_centered() {
        assert_eq!(
            "100x150+0.5+0.5-50%-50%"
                .parse::<LazyRectangle>()
                .unwrap()
                .init(Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: 800.0,
                    height: 600.0,
                }),
            Rectangle {
                x: 400.0 - 50.0,
                y: 300.0 - 75.0,
                width: 100.0,
                height: 150.0
            }
        );
    }

    #[test]
    fn nudge_pushes_to_edge() {
        assert_eq!(
            "100x100+1+1-100%-100%"
                .parse::<LazyRectangle>()
                .unwrap()
                .init(Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: 400.0,
                    height: 400.0,
                }),
            Rectangle {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0
            }
        );
    }

    #[test]
    fn min_max_clamping_with_nudge() {
        assert_eq!(
            "20x20+1+1+100%+100%"
                .parse::<LazyRectangle>()
                .unwrap()
                .init(Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 100.0,
                }),
            Rectangle {
                x: 21.0,
                y: 21.0,
                width: 20.0,
                height: 20.0
            }
        );
    }

    #[test]
    fn invalid_format_missing_components() {
        "100x200+10".parse::<LazyRectangle>().unwrap_err();
        "100x200".parse::<LazyRectangle>().unwrap_err();
        "100x+10+10".parse::<LazyRectangle>().unwrap_err();
        "x200+10+10".parse::<LazyRectangle>().unwrap_err();
        "100x200+10+20+invalid"
            .parse::<LazyRectangle>()
            .unwrap_err();
        "abcx100+10+10".parse::<LazyRectangle>().unwrap_err();
        "100x...!+10+10".parse::<LazyRectangle>().unwrap_err();
        "100x100+X+10".parse::<LazyRectangle>().unwrap_err();
        "100x100+10+Y".parse::<LazyRectangle>().unwrap_err();
        "100x100+0+0-12.5%".parse::<LazyRectangle>().unwrap_err();
        "100x100+0+0-0.5%".parse::<LazyRectangle>().unwrap_err();
        "100x100+0+0-abc%".parse::<LazyRectangle>().unwrap_err();
        "100x100+0+0-50%%".parse::<LazyRectangle>().unwrap_err();
        "100x100+0+0-10%+20%-30%"
            .parse::<LazyRectangle>()
            .unwrap_err();
        "100x100+50+50-50+50".parse::<LazyRectangle>().unwrap_err();
    }

    #[test]
    fn nudge_zero_percent() {
        assert_eq!(
            "100x150+10+20-0%+0%"
                .parse::<LazyRectangle>()
                .unwrap()
                .init(Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: 800.0,
                    height: 600.0,
                }),
            Rectangle {
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 150.0,
            }
        );
    }

    #[test]
    fn nudge_100_percent() {
        assert_eq!(
            "100x150+50+50-100%+100%"
                .parse::<LazyRectangle>()
                .unwrap()
                .init(Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: 800.0,
                    height: 600.0,
                }),
            Rectangle {
                y: 200.0,
                height: 150.0,
                x: 0.0,
                width: 50.0,
            }
        );
    }

    #[test]
    fn single_x_nudge_positive() {
        assert_eq!(
            "100x100+10+20+50%"
                .parse::<LazyRectangle>()
                .unwrap()
                .init(Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: 800.0,
                    height: 600.0,
                }),
            Rectangle {
                x: 10.0 + 0.5 * 100.0,
                y: 20.0,
                width: 100.0,
                height: 100.0,
            }
        );
    }

    #[test]
    fn single_x_nudge_negative() {
        assert_eq!(
            "100x100+50+20-30%"
                .parse::<LazyRectangle>()
                .unwrap()
                .init(Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: 800.0,
                    height: 600.0,
                }),
            Rectangle {
                x: 50.0 - (0.3 * 100.0),
                y: 20.0,
                height: 100.0,
                width: 100.0,
            }
        );
    }

    #[test]
    fn y_coord_is_relative_with_nudges() {
        assert_eq!(
            "100x150+10+0.5-10%+20%"
                .parse::<LazyRectangle>()
                .unwrap()
                .init(Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: 800.0,
                    height: 600.0,
                }),
            Rectangle {
                x: 10.0 - (0.1 * 100.0),
                y: (0.5 * 600.0) + (0.2 * 150.0),
                width: 100.0,
                height: 150.0,
            }
        );
    }

    #[test]
    fn init_with_zero_bounds_relative_rect() {
        assert_eq!(
            "0.5x0.5+0.1+0.1"
                .parse::<LazyRectangle>()
                .unwrap()
                .init(Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: 0.0,
                    height: 0.0,
                }),
            Rectangle {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            }
        );
    }

    #[test]
    fn init_with_zero_bounds_and_nudges() {
        assert_eq!(
            "10x20+0.1+0.1-10%+10%"
                .parse::<LazyRectangle>()
                .unwrap()
                .init(Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: 0.0,
                    height: 0.0,
                }),
            Rectangle {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0
            }
        );
    }

    #[test]
    fn init_nudge_with_zero_width_height_rect_on_non_zero_bounds() {
        assert_eq!(
            "0x0+10+10-50%-50%"
                .parse::<LazyRectangle>()
                .unwrap()
                .init(Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 100.0,
                }),
            Rectangle {
                x: 10.0,
                y: 10.0,
                width: 0.0,
                height: 0.0,
            }
        );
    }

    #[test]
    fn completely_out_of_bounds_rect_is_clamped() {
        assert_eq!(
            "10x10+1000+1000"
                .parse::<LazyRectangle>()
                .unwrap()
                .init(Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 100.0,
                }),
            Rectangle {
                x: 100.0,
                y: 100.0,
                width: 0.0,
                height: 0.0
            }
        );
    }
}
