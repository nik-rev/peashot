//! This module declares all of the theme keys that can be used in the app
//!
//! All theme keys are stored in a flat format for ease of use.

use std::collections::HashMap;

use ferrishot_knus::{DecodeScalar, ast::Literal, errors::DecodeError, traits::ErrorSpan};

/// A color can either be a hex, or it can reference a hex in the `palette` field
///
/// ```kdl
/// theme {
///   palette {
///     black 0x00_00_00
///   }
///   color @black
/// }
/// ```
#[derive(Debug)]
pub enum ColorValue {
    /// A hex color like `0xff_00_00`
    Hex(u32),
    /// References a value in the hashmap by its `name`
    Palette(String),
}

impl<S: ErrorSpan> DecodeScalar<S> for ColorValue {
    fn type_check(
        _type_name: &Option<ferrishot_knus::span::Spanned<ferrishot_knus::ast::TypeName, S>>,
        _ctx: &mut ferrishot_knus::decode::Context<S>,
    ) {
    }

    fn raw_decode(
        value: &ferrishot_knus::span::Spanned<Literal, S>,
        ctx: &mut ferrishot_knus::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        match &**value {
            Literal::Int(int) => match int.try_into() {
                Ok(v) => Ok(Self::Hex(v)),
                Err(err) => {
                    ctx.emit_error(DecodeError::conversion(value, err));
                    Ok(Self::Hex(0))
                }
            },
            Literal::String(s) => Ok(Self::Palette(s.to_string())),
            _ => {
                ctx.emit_error(DecodeError::scalar_kind(
                    ferrishot_knus::decode::Kind::String,
                    value,
                ));
                Ok(Self::Hex(0))
            }
        }
    }
}

/// Represents the color node used in the KDL config, to be parsed into
/// this structure.
///
/// # Examples
///
/// ```kdl
/// theme {
///   // an opaque white color
///   background ffffff
///   // black color with 50% opacity
///   foreground 000000 opacity=0.5
/// }
/// ```
#[derive(ferrishot_knus::Decode, Debug)]
pub struct Color {
    /// Hex color. Examples:
    ///
    /// - `ff0000`: Red
    /// - `000000`: Black
    #[ferrishot_knus(argument)]
    pub color: ColorValue,
    /// The opacity for this color.
    /// - `1.0`: Opaque
    /// - `0.0`: Transparent
    #[ferrishot_knus(default = 1.0, property)]
    pub opacity: f32,
}

/// Declare theme keys
///
/// `UserKdlTheme` is merged into `DefaultKdlTheme` before being processed
/// into a `Theme`
#[macro_export]
macro_rules! declare_theme_options {
    (
        $(
            $(#[$doc:meta])*
            $key:ident
        ),* $(,)?
    ) => {
        /// Theme and colors of ferrishot
        #[derive(Debug, Copy, Clone)]
        pub struct Theme {
            $(
                $(#[$doc])*
                pub $key: iced::Color,
            )*
        }

        /// Ferrishot's default theme and colors
        #[derive(ferrishot_knus::Decode, Debug)]
        pub struct DefaultKdlTheme {
            /// Palette
            #[ferrishot_knus(child, unwrap(properties))]
            palette: HashMap<String, u32>,
            $(
                $(#[$doc])*
                #[ferrishot_knus(child)]
                pub $key: Color,
            )*
        }

        /// The user's custom theme and color overrides
        /// All values are optional and will override whatever is the default
        #[derive(ferrishot_knus::Decode, Debug)]
        pub struct UserKdlTheme {
            /// Palette
            #[ferrishot_knus(child, unwrap(properties))]
            palette: Option<HashMap<String, u32>>,
            $(
                $(#[$doc])*
                #[ferrishot_knus(child)]
                pub $key: Option<$crate::config::Color>,
            )*
        }

        impl DefaultKdlTheme {
            /// If the user theme specifies a color, it will override the color in the
            /// default theme.
            pub fn merge_user_theme(mut self, user_theme: UserKdlTheme) -> Self {
                // merge the palette
                if let Some(palette) = user_theme.palette {
                    self.palette.extend(palette.into_iter());
                }
                // merge rest of the keys
                $(
                    self.$key = user_theme.$key.unwrap_or(self.$key);
                )*
                self
            }
        }

        impl TryFrom<DefaultKdlTheme> for Theme {
            type Error = String;

            fn try_from(value: DefaultKdlTheme) -> Result<Self, Self::Error> {
                Ok(Self {
                    $(
                        $key: {
                            let hex = match value.$key.color {
                                ColorValue::Hex(hex) => hex,
                                ColorValue::Palette(key) =>
                                    *value.palette
                                        .get(&key)
                                        .ok_or_else(
                                            || format!("This palette item does not exist: {key}")
                                        )?
                            };
                            let [.., r, g, b] = hex.to_be_bytes();

                            iced::Color::from_rgba(
                                f32::from(r) / 255.0,
                                f32::from(g) / 255.0,
                                f32::from(b) / 255.0,
                                value.$key.opacity
                            )
                        },
                    )*
                })
            }
        }
    }
}

crate::declare_theme_options! {
    /// Cheatsheet background
    cheatsheet_bg,
    /// Cheatsheet text color
    cheatsheet_fg,

    /// Close the popup
    popup_close_icon_bg,
    /// Cheatsheet text color
    popup_close_icon_fg,

    /// Color of the border around the selection
    selection_frame,
    /// Color of the region outside of the selected area
    non_selected_region,
    /// Color of drop shadow, used for stuff like:
    ///
    /// - drop shadow of icons
    /// - drop shadow of selection rectangle
    /// - drop shadow around error box
    drop_shadow,
    /// Background color of selected text
    text_selection,

    //
    // --- Side Indicator ---
    //
    /// Foreground color of the size indicator
    size_indicator_fg,
    /// Background color of the size indicator
    size_indicator_bg,

    //
    // --- Tooltip ---
    //
    /// Text color of the tooltip
    tooltip_fg,
    /// Background color of the tooltip
    tooltip_bg,

    //
    // --- Errors ---
    //
    /// Color of the text on errors
    error_fg,
    /// Background color of the error boxes
    error_bg,

    //
    // --- Info Box ---
    //
    /// Background color of the info box, which shows various tips
    info_box_bg,
    /// Text color of the info box, which shows various tips
    info_box_fg,
    /// Color of the border of the info box
    info_box_border,

    //
    // --- Selection Icons ---
    //
    /// Background color of the icons around the selection
    icon_bg,
    /// Color of icons around the selection
    icon_fg,

    //
    // --- Debug Menu ---
    //
    /// Color of the labels in the debug menu (F12)
    debug_label,
    /// Foreground color of debug menu (F12)
    debug_fg,
    /// Background color of debug menu (F12)
    debug_bg,

    //
    // --- Letters ---
    //
    /// Color of lines
    letters_lines,
    /// Color of letters
    letters_fg,
    /// Background color of letters
    letters_bg,

    //
    // --- Image uploaded popup ---
    //
    /// Foreground color of the image_uploaded popup
    image_uploaded_fg,
    /// Background color of the image_uploaded popup
    image_uploaded_bg,

    /// Color of success, e.g. green check mark when copying text to clipboard
    success,
}
