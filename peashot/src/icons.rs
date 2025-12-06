//! Icons for ferrishot
//!
//! - Icons are stored in the `assets/icons/` directory.
//! - Icons are declared at the invocation of the `icons!` macro.
//! - Each `Icon` must have a corresponding `icons/Icon.svg` file.

/// Generates handles for macros and automatically includes all the icons
macro_rules! load_icons {
    (
        $(
            #[$doc:meta]
            $icon:ident
        ),* $(,)?
    ) => {
        /// Icons for ferrishot
        #[expect(dead_code, reason = "not all icons are used at the moment")]
        #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
        pub enum Icon {
            $(
                #[$doc]
                $icon
            ),*
        }

        /// Private module so we don't leak implementation detail of the static icons
        mod __static_icons {
            $(
                #[expect(nonstandard_style, reason = "handy for creating statics")]
                pub(super) static $icon: std::sync::LazyLock<iced::widget::svg::Handle> = std::sync::LazyLock::new(|| {
                    iced::widget::svg::Handle::from_memory(include_bytes!(concat!(
                        "../assets/icons/",
                        stringify!($icon),
                        ".svg"
                    )))
                });
            )*

        }

        impl Icon {
            /// Obtain this icon's svg handle
            pub fn svg(self) -> iced::widget::svg::Handle {
                match self {
                    $(Self::$icon => __static_icons::$icon.clone()),*
                }
            }
        }
    }
}

load_icons! {
    /// Arrow pointing up
    ArrowUp,
    /// Arrow pointing right
    ArrowRight,
    /// Arrow pointing down
    ArrowDown,
    /// Arrow pointing left
    ArrowLeft,
    /// Save the image to a path by opening the file dialog
    Save,
    /// Drawing a circle
    Circle,
    /// Copy the image to clipboard
    Clipboard,
    /// Close the app
    Close,
    /// Switch to Cursor tool, allows resizing and dragging the selection around
    Cursor,
    /// Select the entire image
    Fullscreen,
    /// Draw on the image
    Pen,
    /// Draw a square
    Square,
    /// Add text
    Text,
    /// Upload image to the internet
    Upload,
    /// Indicate success
    Check,
    /// Loading...
    Spinner,
}

/// Expands to an SVG by reading from the `icons/` directory
#[macro_export]
macro_rules! icon {
    ($icon:ident) => {{ iced::widget::svg($crate::icons::Icon::$icon.svg()) }};
}
