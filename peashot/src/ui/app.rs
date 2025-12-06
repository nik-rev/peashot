//! Main logic for the application, handling of events and mutation of the state

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use crate::Cli;
use crate::Config;
use crate::image::RgbaHandle;
use crate::image::action::ImageData;
use crate::message::Message;
use crate::ui;
use crate::ui::popup;
use iced::Length::Fill;
use iced::Renderer;
use iced::Subscription;
use iced::Task;
use iced::Theme;
use iced::mouse::Interaction;
use iced::widget::Canvas;
use iced::widget::Stack;
use iced::window;
use iced::{
    Rectangle,
    widget::{Action, canvas},
};
use image::DynamicImage;
use indoc::formatdoc;
use tap::Pipe as _;

use crate::geometry::RectangleExt as _;
use crate::ui::selection::Selection;

use super::Errors;
use super::popup::Popup;
use super::selection::OptionalSelectionExt as _;
use super::selection::SelectionKeysState;

crate::declare_commands! {
    enum Command {
        /// Do nothing
        NoOp,
        /// Exit the application
        Exit,
    }
}

impl crate::command::Handler for Command {
    fn handle(self, _app: &mut App, _count: u32) -> Task<Message> {
        match self {
            Self::NoOp => Task::none(),
            Self::Exit => App::exit(),
        }
    }
}

/// Holds the state for ferrishot
#[derive(Debug)]
pub struct App {
    /// If an image is in the process of being uploaded (but hasn't yet)
    pub is_uploading_image: bool,
    /// When the application was launched
    pub time_started: Instant,
    /// How long has passed since starting ferrishot
    pub time_elapsed: Duration,
    /// Config of the app
    pub config: Arc<Config>,
    /// A list of messages which obtained while the debug overlay is active
    pub logged_messages: Vec<Message>,
    /// How many selections were created throughout the
    /// lifetime of the App
    pub selections_created: usize,
    /// The full screenshot of the monitor from which ferrishot was invoked
    /// We then create a window spanning the entire monitor, with this
    /// screenshot as background, with a canvas rendered on top - giving the
    /// illusion that we are drawing shapes on top of the screen.
    pub image: Arc<RgbaHandle>,
    /// Area of the screen that is selected for capture
    pub selection: Option<Selection>,
    /// Errors to display to the user
    pub errors: Errors,
    /// Whether to show an overlay with additional information (F12)
    pub show_debug_overlay: bool,
    /// Command line arguments passed
    pub cli: Arc<Cli>,

    /// Currently opened popup
    pub popup: Option<Popup>,
}

#[bon::bon]
impl App {
    /// Run the `app` in headless mode. So, simply do whatever action is necessary and do not spawn a window
    ///
    /// Returns a closure which takes path of the saved image. It has to be this way because we don't
    /// actually know where the image will be saved until the end of `main`.
    pub async fn headless(
        action: crate::image::action::Command,
        region: Rectangle,
        image: Arc<RgbaHandle>,
        is_json: bool,
    ) -> Result<Box<dyn Fn(Option<PathBuf>) -> String>, crate::image::action::Error> {
        use crate::image::action::Output as O;

        let (output, ImageData { height, width }) = image
            .pipe(|img| Self::process_image(region, &img))
            .pipe(|img| action.execute(img, region))
            .await?;

        let green = anstyle::AnsiColor::Green
            .on_default()
            .effects(anstyle::Effects::BOLD);
        let reset = anstyle::Reset;

        let tick = format!("{green}✓{reset}");

        let closure: Box<dyn Fn(Option<PathBuf>) -> String> = match output {
            O::Saved => Box::new(move |saved_path| {
                let save_path = saved_path
                    .as_ref()
                    .map(|path| format!("{}", path.display()))
                    .unwrap_or_default();

                let file_size_bytes = saved_path
                    .unwrap_or_default()
                    .metadata()
                    .map(|meta| meta.len())
                    .unwrap_or(0);

                let file_size = human_bytes::human_bytes(file_size_bytes as f64);

                if is_json {
                    formatdoc! {
                        r#"
                            {{
                                "type": "save",
                                "width": {width},
                                "height": {height},
                                "fileSize": "{file_size}",
                                "fileSizeInBytes": {file_size_bytes},
                                "savePath": "{save_path}"
                            }}
                        "#
                    }
                } else {
                    formatdoc! {
                        "
                            {tick} Image saved to {save_path}

                            width: {width} px
                            height: {height} px
                            file size: {file_size}
                        ",
                    }
                }
            }),
            O::Copied => Box::new(move |_| {
                if is_json {
                    formatdoc! {
                        r#"
                            {{
                                "type": "copy",
                                "width": {width},
                                "height": {height},
                            }}
                        "#
                    }
                } else {
                    formatdoc! {
                        "
                            {tick} Image copied to clipboard

                            width: {width} px
                            height: {height} px
                        "
                    }
                }
            }),
            O::Uploaded {
                data,
                file_size: file_size_bytes,
                ..
            } => Box::new(move |_| {
                let link = &data.link;
                let expires = data.expires_in;
                let file_size = human_bytes::human_bytes(file_size_bytes as f64);

                if is_json {
                    formatdoc! {
                        r#"
                            {{
                                "type": "upload",
                                "width": {width},
                                "height": {height},
                                "fileSize": "{file_size}",
                                "fileSizeInBytes": {file_size_bytes},
                                "link": "{link}",
                                "expiresIn": "{expires}"
                            }}
                        "#
                    }
                } else {
                    formatdoc! {
                        "
                            {tick} Image uploaded to {link}

                            width: {width} px
                            height: {height} px
                            file size: {file_size}
                            expires in: {expires}
                        "
                    }
                }
            }),
        };

        Ok(closure)
    }

    /// Create a new `App`
    #[builder]
    pub fn new(
        cli: Arc<Cli>,
        config: Arc<Config>,
        initial_region: Option<Rectangle>,
        image: Arc<RgbaHandle>,
    ) -> Self {
        Self {
            is_uploading_image: false,
            time_started: Instant::now(),
            time_elapsed: Duration::ZERO,
            selection: initial_region.map(|rect| Selection {
                is_first: true,
                accept_on_select: cli.accept_on_select,
                theme: config.theme,
                rect,
                status: ui::selection::SelectionStatus::default(),
            }),
            logged_messages: vec![],
            selections_created: 0,
            // FIXME: Currently the app cannot handle when the resolution is very small
            // if a path was passed and the path contains a valid image
            image,
            errors: Errors::default(),
            show_debug_overlay: cli.debug,
            config,
            cli,
            popup: None,
        }
    }

    /// Close the app
    ///
    /// This is like `iced::exit`, but it does not cause a segfault in special
    /// circumstances <https://github.com/iced-rs/iced/issues/2625>
    ///
    /// # Panics
    ///
    /// If there is no window
    pub fn exit() -> Task<Message> {
        window::get_latest().then(|id| window::close(id.expect("window to exist")))
    }

    /// This method is used to keep track of time / how much time has passed since start
    /// of the program, using this for animations.
    pub fn subscription(&self) -> Subscription<Message> {
        window::frames().map(Message::Tick)
    }

    /// Renders the app
    pub fn view(&self) -> iced::Element<Message> {
        Stack::new()
            // taken screenshot in the background
            .push(super::BackgroundImage {
                image_handle: RgbaHandle::clone(&self.image).into(),
            })
            // Shade in the background + global event handler + selection renderer
            .push(Canvas::new(self).width(Fill).height(Fill))
            // information popup with basic tips
            .push_maybe(
                (self.popup.is_none() && self.selection.is_none())
                    .then(|| super::welcome_message(self)),
            )
            // errors
            .push(self.errors.view(self))
            // icons around the selection
            .push_maybe(
                self.selection
                    .filter(|sel| sel.is_idle() && self.config.selection_icons)
                    .map(|sel| {
                        super::SelectionIcons {
                            app: self,
                            image_width: self.image.width() as f32,
                            image_height: self.image.height() as f32,
                            selection_rect: sel.rect.norm(),
                        }
                        .view()
                    }),
            )
            // size indicator
            .push_maybe(
                self.selection
                    .filter(|_| self.config.size_indicator)
                    .get()
                    .map(|(sel, sel_is_some)| {
                        super::size_indicator(self, sel.rect.norm(), sel_is_some)
                    }),
            )
            .push_maybe(self.popup.as_ref().map(|popup| {
                match popup {
                    Popup::Letters(state) => popup::Letters {
                        app: self,
                        pick_corner: state.picking_corner,
                    }
                    .view(),
                    Popup::ImageUploaded(state) => popup::ImageUploaded {
                        app: self,
                        qr_code_data: &state.url.0,
                        data: &state.url.1,
                        url_copied: state.has_copied_link,
                    }
                    .view(),
                    Popup::KeyCheatsheet => popup::KeybindingsCheatsheet {
                        theme: &self.config.theme,
                    }
                    .view(),
                }
            }))
            // debug overlay
            .push_maybe(self.show_debug_overlay.then(|| super::debug_overlay(self)))
            .into()
    }

    /// Convert the image into its final form, with crop (and in the future will also have
    /// "decorations" such as arrow, circle, square)
    ///
    /// # Panics
    ///
    /// The stored image is not a valid RGBA image
    pub fn process_image(rect: Rectangle, image: &RgbaHandle) -> DynamicImage {
        DynamicImage::from(
            image::RgbaImage::from_raw(image.width(), image.height(), image.bytes().to_vec())
                .expect("Image handle stores a valid image"),
        )
        .crop_imm(
            rect.x as u32,
            rect.y as u32,
            rect.width as u32,
            rect.height as u32,
        )
    }

    /// Modifies the app's state
    pub fn update(&mut self, message: Message) -> Task<Message> {
        use crate::message::Handler as _;

        match message {
            Message::Exit => return Self::exit(),
            Message::ClosePopup => {
                self.popup = None;
            }
            Message::Tick(instant) => {
                self.time_elapsed = instant.duration_since(self.time_started);
            }
            Message::KeyCheatsheet(key_cheatsheet) => {
                return key_cheatsheet.handle(self);
            }
            Message::Selection(selection) => {
                return selection.handle(self);
            }
            Message::SizeIndicator(size_indicator) => {
                return size_indicator.handle(self);
            }
            Message::ImageUploaded(image_uploaded) => {
                return image_uploaded.handle(self);
            }
            Message::Letters(letters) => {
                return letters.handle(self);
            }
            Message::NoOp => (),
            Message::Command { action, count } => {
                return <crate::Command as crate::command::Handler>::handle(action, self, count);
            }
            Message::Error(err) => {
                self.errors.push(err);
            }
        }

        Task::none()
    }
}

/// Holds information about the mouse
#[derive(Default, Debug, Clone)]
pub struct AppKeysState {
    /// Left mouse click is currently being held down
    pub is_left_down: bool,
    /// How many times to execute the next motion
    pub motion_count: Option<u32>,
    /// The last key that was pressed
    pub last_key_pressed: Option<iced::keyboard::Key>,
}

impl canvas::Program<Message> for App {
    type State = (AppKeysState, SelectionKeysState);

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        if let Some(sel) = self.selection.map(Selection::norm) {
            sel.draw(&mut frame, bounds);
        } else {
            // usually the selection is responsible for drawing shade around itself
            // However here we don't have selection, so just draw the shade on the entire screen
            frame.fill_rectangle(
                bounds.position(),
                bounds.size(),
                self.config.theme.non_selected_region,
            );
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: iced::advanced::mouse::Cursor,
    ) -> Option<Action<Message>> {
        use iced::Event::{Keyboard, Mouse, Touch};
        use iced::keyboard::Event::KeyPressed;
        use iced::keyboard::Key::Named;
        use iced::keyboard::Modifiers;
        use iced::keyboard::key::Named::{ArrowDown, ArrowLeft, ArrowRight, ArrowUp, Shift};
        use iced::mouse::Button::Left;
        use iced::mouse::Event::ButtonPressed;
        use iced::mouse::Event::ButtonReleased;
        use iced::touch::Event::{FingerLifted, FingerPressed};

        // Handle popups. Esc = close popup
        //
        // Events will still be forwarded to the canvas even if we have a popup
        if self.popup.is_some() {
            if let Keyboard(KeyPressed {
                key: Named(iced::keyboard::key::Named::Escape),
                ..
            }) = event
            {
                return Some(Action::publish(Message::ClosePopup));
            }

            return None;
        }

        let (state, selection_state) = state;

        if let Some(sel) = self.selection {
            if let Some(action) = sel.update(selection_state, event, bounds, cursor) {
                return Some(action);
            }
        }

        // handle the number pressed
        //
        // pressing numbers will have an effect, e.g. `200j` will
        // move the selection down by 200px
        if let Keyboard(KeyPressed {
            key: iced::keyboard::Key::Character(ch),
            ..
        }) = event
        {
            if let Ok(number_pressed) = ch.parse::<u32>() {
                if let Some(motion_count) = state.motion_count.as_mut() {
                    *motion_count = *motion_count * 10 + number_pressed;
                } else {
                    state.motion_count = Some(number_pressed);
                }
            }
        }

        // handle keybindings
        if let Keyboard(KeyPressed {
            modifiers,
            modified_key,
            key,
            ..
        }) = event
        {
            let mut modifiers = *modifiers;

            // Shift key does not matter. For example:
            // - pressing `<` and the `SHIFT` modifier will be pressed
            // - `G` will also trigger the `SHIFT` modifier
            //
            // However, we are going to hard-code the shift modifier to not be removed for the
            // arrow keys
            if !matches!(key, Named(ArrowLeft | ArrowDown | ArrowRight | ArrowUp)) {
                modifiers.remove(Modifiers::SHIFT);
            }

            if let Some(action) = state
                .last_key_pressed
                .as_ref()
                .and_then(|last_key_pressed| {
                    self.config.keys.get(
                        last_key_pressed.clone(),
                        Some(modified_key.clone()),
                        modifiers,
                    )
                })
                .or_else(|| self.config.keys.get(modified_key.clone(), None, modifiers))
            {
                // the last key pressed needs to be reset for it to be
                // correct in future invocations
                //
                // For example if I press `gg`, and it activates some keybinding
                // I would have to press `gg` *again* to active it.
                //
                // If we did not reset, then `ggg` would trigger the `gg` keybindings
                // twice
                state.last_key_pressed = None;

                let count = state.motion_count.unwrap_or(1);
                state.motion_count = None;

                return Some(Action::publish(Message::Command {
                    action: action.clone(),
                    count,
                }));
            }

            // the "Shift" is already included in the modifiers
            //
            // Otherwise, when pressing 'G' for instance it would first set
            // - `last_key_pressed = Shift` then
            // - `last_key_pressed = 'G'`
            if *modified_key != Named(Shift) {
                state.last_key_pressed = Some(modified_key.clone());
            }
        }

        // Create the selection when it does not exist yet

        let message = match event {
            Touch(FingerPressed { .. }) | Mouse(ButtonPressed(Left)) => {
                state.is_left_down = true;
                Message::Selection(Box::new(ui::selection::Message::CreateSelection(
                    cursor.position()?,
                )))
            }
            Touch(FingerLifted { .. }) | Mouse(ButtonReleased(Left)) => {
                state.is_left_down = false;
                Message::NoOp
            }
            _ => return None,
        };

        Some(Action::publish(message))
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        cursor: iced::advanced::mouse::Cursor,
    ) -> Interaction {
        if let Some(Popup::ImageUploaded(_)) = self.popup {
            Interaction::default()
        } else {
            self.selection
                .map(Selection::norm)
                .map_or(Interaction::Crosshair, |sel| sel.mouse_interaction(cursor))
        }
    }
}
