//! Widgets with custom styles

use iced::Element;

pub mod app;
mod background_image;
pub mod debug_overlay;
mod errors;
mod grid;
mod selection_icons;
mod welcome_message;

pub mod selection;

use background_image::BackgroundImage;
use debug_overlay::debug_overlay;
use errors::Errors;

pub mod popup;

pub mod size_indicator;
use size_indicator::size_indicator;

use selection_icons::SelectionIcons;
use welcome_message::welcome_message;

pub use app::App;

/// An extension trait to show a red border around an element and all children
#[easy_ext::ext(Explainer)]
#[expect(
    clippy::allow_attributes,
    reason = "so we dont have to switch between expect/allow"
)]
#[allow(dead_code, reason = "useful to exist for debugging")]
pub impl<'a, M: 'a, E> E
where
    E: Into<Element<'a, M>>,
{
    /// Shows red border around an element and all of its children
    fn explain(self) -> Element<'a, M> {
        self.into().explain(iced::Color::from_rgb8(255, 0, 0))
    }
}
