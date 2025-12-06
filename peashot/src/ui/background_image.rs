//! Renders the full, unprocessed desktop screenshot on the screen
use iced::Length::Fill;
use iced::advanced::widget::Tree;
use iced::advanced::{Layout, Widget, layout, renderer};
use iced::widget::image;
use iced::{Element, Length, Rectangle, Size, Theme, mouse};

#[derive(Debug)]
/// A widget that draws an image on the entire screen
pub struct BackgroundImage {
    /// Image handle of the full-desktop screenshot
    pub image_handle: image::Handle,
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for BackgroundImage
where
    Renderer: iced::advanced::Renderer + iced::advanced::image::Renderer<Handle = image::Handle>,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Fill,
            height: Fill,
        }
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        image::layout(
            renderer,
            limits,
            &self.image_handle,
            Fill,
            Fill,
            iced::ContentFit::Contain,
            iced::Rotation::Solid(0.into()),
        )
    }

    fn draw(
        &self,
        _state: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        image::draw(
            renderer,
            layout,
            viewport,
            &self.image_handle,
            iced::ContentFit::Contain,
            image::FilterMethod::Nearest,
            iced::Rotation::Solid(0.into()),
            1.0,
            1.0,
        );
    }
}

impl<Message, Renderer> From<BackgroundImage> for Element<'_, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer + iced::advanced::image::Renderer<Handle = image::Handle>,
{
    fn from(widget: BackgroundImage) -> Self {
        Self::new(widget)
    }
}
