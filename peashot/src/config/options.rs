//! Declare the flat, top-level config options of ferrishot
//!
//! This module touches
//!
//! ```kdl
//!
//! ```

/// Declare config options
///
/// `UserKdlConfig` is merged into `DefaultKdlConfig` before being processed
/// into a `Config`
#[macro_export]
macro_rules! declare_config_options {
    (
        $(#[$ConfigAttr:meta])*
        struct $Config:ident {
            $(#[$keys_doc:meta])*
            $keys:ident: $Keys:ty,
            $(#[$theme_doc:meta])*
            $theme:ident: $Theme:ty,
            $(
                $(#[$doc:meta])*
                $key:ident: $typ:ty
            ),* $(,)?
        }
    ) => {
        $(#[$ConfigAttr])*
        pub struct $Config {
            $(#[$theme_doc])*
            pub $theme: $Theme,
            $(#[$keys_doc])*
            pub $keys: $Keys,
            $(
                $(#[$doc])*
                pub $key: $typ,
            )*
        }

        /// The default config as read from the default config file, included as a static string in the binary.
        /// All values are required and must be specified
        #[derive(ferrishot_knus::Decode, Debug)]
        pub struct DefaultKdlConfig {
            /// The default keybindings of ferrishot
            #[ferrishot_knus(child)]
            pub $keys: $crate::config::key_map::Keys,
            /// The default theme of ferrishot
            #[ferrishot_knus(child)]
            pub $theme: super::theme::DefaultKdlTheme,
            $(
                $(#[$doc])*
                #[ferrishot_knus(child, unwrap(argument))]
                pub $key: $typ,
            )*
        }

        impl DefaultKdlConfig {
            /// Merge the user's top-level config options with the default options.
            /// User config options take priority.
            pub fn merge_user_config(mut self, user_config: UserKdlConfig) -> Self {
                $(
                    self.$key = user_config.$key.unwrap_or(self.$key);
                )*
                // merge keybindings
                //
                // If the same keybinding is defined in the default theme and
                // the user theme, e.g.
                //
                // default:
                //
                // ```kdl
                // keys {
                //   goto top-left key=gg
                // }
                // ```
                //
                // user:
                //
                // ```kdl
                // keys {
                //   goto bottom-right key=gg
                // }
                // ```
                //
                // The user's keybinding will come after. Since we are iterating over the
                // keys sequentially, and inserting into the `KeyMap` one-by-one, the default keybinding
                // will be inserted into the `KeyMap`, but it will be overridden by the user keybinding.
                //
                // Essentially what we want to make sure is that if the same key is defined twice,
                // the user keybinding takes priority.
                self
                    .keys
                    .keys
                    .extend(user_config.keys.unwrap_or_default().keys);

                if let Some(user_theme) = user_config.theme {
                    self.theme = self.theme.merge_user_theme(user_theme);
                };

                self
            }
        }

        impl TryFrom<DefaultKdlConfig> for $Config {
            type Error = String;

            fn try_from(value: DefaultKdlConfig) -> Result<Self, Self::Error> {
                Ok(Self {
                    $(
                        $key: value.$key,
                    )*
                    theme: value.theme.try_into()?,
                    keys: value.keys.keys.into_iter().collect::<$crate::config::KeyMap>(),
                })
            }
        }

        /// User's config. Everything is optional. Values will be merged with `DefaultKdlConfig`.
        /// And will take priority over the default values.
        #[derive(ferrishot_knus::Decode, Debug)]
        pub struct UserKdlConfig {
            /// User-defined keybindings
            #[ferrishot_knus(child)]
            pub keys: Option<$crate::config::key_map::Keys>,
            /// User-defined colors
            #[ferrishot_knus(child)]
            pub theme: Option<super::theme::UserKdlTheme>,
            $(
                $(#[$doc])*
                #[ferrishot_knus(child, unwrap(argument))]
                pub $key: Option<$typ>,
            )*
        }
    }
}

crate::declare_config_options! {
    /// Configuration for ferrishot.
    #[derive(Debug)]
    struct Config {
        /// Ferrishot's keybindings
        keys: super::key_map::KeyMap,
        /// Ferrishot's theme and colors
        theme: super::Theme,
        /// Renders a size indicator in the bottom left corner.
        /// It shows the current height and width of the selection.
        ///
        /// You can manually enter a value to change the selection by hand.
        size_indicator: bool,
        /// Render icons around the selection
        selection_icons: bool,
    }
}
