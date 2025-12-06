//! Parse user keybindings

use crate::config::named_key::Named;
use std::{collections::HashMap, str::FromStr};

use iced::{
    advanced::debug::core::SmolStr,
    keyboard::{Modifiers, key::Key as IcedKey},
};

use super::commands::{Command, KeymappableCommand};

/// Represents the keybindings for ferrishot
#[derive(Debug, Default)]
pub struct KeyMap {
    /// Map of Key Pressed => Action when pressing that key
    pub keys: HashMap<(KeySequence, KeyMods), Command>,
}

impl KeyMap {
    /// Get a key from the `KeyMap`
    ///
    /// So why do we need owned values here? `.get()` methods returning a reference usually
    /// do not require owned values
    ///
    /// If we had an `&IcedKey` and an `Option<&IcedKey>` we won't be able to turn that into
    /// an `&KeySequence` without cloning unfortunately.
    ///
    /// This is a well-known problem in Rust: <https://stackoverflow.com/questions/45786717/how-to-get-value-from-hashmap-with-two-keys-via-references-to-both-keys/45795699#45795699>
    ///
    /// The solution outlined in the Stack Overflow post, while fun, is not zero-cost and
    /// adds too much unnecessary complexity whilst probably being slower. (We have to go
    /// through the V-Table lookup, as it requires using dynamic dispatch)
    ///
    pub fn get(
        &self,
        key: IcedKey,
        previous_key: Option<IcedKey>,
        mods: Modifiers,
    ) -> Option<&Command> {
        self.keys
            .get(&(KeySequence((key, previous_key)), KeyMods(mods)))
    }
}

/// Keybindings for ferrishot
#[derive(ferrishot_knus::Decode, Debug, Default)]
pub struct Keys {
    /// A list of raw keybindings for ferrishot, directly as read from the config file
    #[ferrishot_knus(children)]
    pub keys: Vec<KeymappableCommand>,
}

impl FromIterator<KeymappableCommand> for KeyMap {
    fn from_iter<T: IntoIterator<Item = KeymappableCommand>>(iter: T) -> Self {
        Self {
            keys: iter.into_iter().map(KeymappableCommand::action).collect(),
        }
    }
}

/// A sequence of 2 keys. If there are 2 keys like so:
/// - (T, None)
/// - (T, Some(X))
///
/// The 2nd key will never be triggered.
/// We will first search the `HashMap` of keys for the first key.
/// If it does not exist, search for the 2nd key.
#[derive(Debug, Hash, PartialEq, PartialOrd, Ord, Eq, Clone)]
pub struct KeySequence(pub (IcedKey, Option<IcedKey>));

/// Modifier keys
#[derive(Debug, Default, Clone, Hash, Eq, PartialEq)]
pub struct KeyMods(pub Modifiers);

impl FromStr for KeyMods {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut mods = Modifiers::empty();
        if s.is_empty() {
            return Ok(Self(Modifiers::empty()));
        }
        for modifier_str in s.split('+') {
            let modifier = match modifier_str.trim() {
                "ctrl" => Modifiers::CTRL,
                "alt" => Modifiers::ALT,
                "super" | "windows" | "command" => Modifiers::LOGO,
                "shift" => Modifiers::SHIFT,
                invalid => return Err(format!("Invalid modifier: {invalid}")),
            };
            if mods.contains(modifier) {
                return Err(format!("Duplicate modifier: {modifier_str}"));
            }
            mods.insert(modifier);
        }

        Ok(Self(mods))
    }
}

impl FromStr for KeySequence {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut keys = vec![];
        // For example, a string like `<<` is valid and means
        // pressing the `<` key twice in a row.
        let mut maybe_parsing_named_key = false;
        let mut named_key_buf = String::new();
        let mut chars = s.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '<' {
                if maybe_parsing_named_key {
                    // we encounter the second `<` in a row
                    //
                    // <<
                    //  x <-- we are here
                    //
                    // that means
                    // the first one was 100% a key.
                    keys.push(IcedKey::Character(SmolStr::new("<")));
                } else {
                    maybe_parsing_named_key = true;
                }

                // SPECIAL-CASE: there is no next character, the strings ends with
                // `<` so it will be a keybinding
                if chars.peek().is_none() {
                    keys.push(IcedKey::Character(SmolStr::new("<")));
                }
            } else if maybe_parsing_named_key {
                if ch == '>' {
                    if named_key_buf.is_empty() {
                        // SPECIAL-CASE: in this case the user types exactly `<>`
                        // Make sure that the first `<` is also not ignored
                        keys.push(IcedKey::Character(SmolStr::new("<")));
                        keys.push(IcedKey::Character(SmolStr::new(">")));
                    } else {
                        // we are currently at the end of a named key
                        //
                        // <space>
                        //       x <-- we are here
                        //
                        // it must be a valid key
                        keys.push(IcedKey::Named(
                            Named::from_str(&named_key_buf)
                                .map_err(|err| format!("Invalid key: <{named_key_buf}>. {err}"))?
                                .to_iced(),
                        ));
                        named_key_buf.clear();
                    }
                    maybe_parsing_named_key = false;
                } else {
                    // we are currently inside of a named key like so
                    //
                    // <space>
                    //   x <-- we may be here
                    named_key_buf.push(ch);
                }
            } else {
                keys.push(IcedKey::Character(SmolStr::new(ch.to_string())));
            }
        }
        let mut keys = keys.into_iter();
        let Some(first_key) = keys.next() else {
            return Err(String::from("Expected at least 1 key."));
        };
        let second_key = keys.next();
        if keys.next().is_some() {
            // because we only store a single previous key, we can't handle keybindings
            // with more than 1 key. Since this is a screenshot app and not something like a
            // text editor, I don't believe there is much utility in allowing 3 keys in a row or more.
            //
            // This greatly simplifies the code, as we don't have to be generic.
            return Err(String::from(
                "At the moment, only up to 2 keys in a sequence are supported.",
            ));
        }
        Ok(Self((first_key, second_key)))
    }
}

#[cfg(test)]
mod test {
    use iced::keyboard::key;

    use super::*;
    use pretty_assertions::assert_eq;

    fn ch(c: &str) -> IcedKey {
        IcedKey::Character(SmolStr::new(c))
    }

    #[track_caller]
    fn parse(input: &str, expected: Result<KeySequence, String>) {
        assert_eq!(
            input.parse::<KeySequence>(),
            expected,
            "Failed to parse {:?}",
            input
        );
    }

    #[test]
    fn parse_key_sequence() {
        use IcedKey::Named;
        use key::Named::*;

        parse("gh", Ok(KeySequence((ch("g"), Some(ch("h"))))));
        parse("ge", Ok(KeySequence((ch("g"), Some(ch("e"))))));
        parse("x", Ok(KeySequence((ch("x"), None))));
        parse("Lx", Ok(KeySequence((ch("L"), Some(ch("x"))))));
        parse("", Err("Expected at least 1 key.".to_string()));
        parse("<space>x", Ok(KeySequence((Named(Space), Some(ch("x"))))));
        parse("x<space>", Ok(KeySequence((ch("x"), Some(Named(Space))))));
        parse(
            "<space><space>",
            Ok(KeySequence((Named(Space), Some(Named(Space))))),
        );
        parse("<<", Ok(KeySequence((ch("<"), Some(ch("<"))))));
        parse("<>", Ok(KeySequence((ch("<"), Some(ch(">"))))));
        parse("<", Ok(KeySequence((ch("<"), None))));
        parse(">>", Ok(KeySequence((ch(">"), Some(ch(">"))))));
        parse("<<space>", Ok(KeySequence((ch("<"), Some(Named(Space))))));
        parse(
            "<f32><f31>",
            Ok(KeySequence((Named(F32), Some(Named(F31))))),
        );
        parse("><f32>", Ok(KeySequence((ch(">"), Some(Named(F32))))));
        parse(
            "abc",
            Err("At the moment, only up to 2 keys in a sequence are supported.".to_string()),
        );
        parse(
            "<f32>b<f16>",
            Err("At the moment, only up to 2 keys in a sequence are supported.".to_string()),
        );
        parse(
            "<@>",
            Err("Invalid key: <@>. Matching variant not found".to_string()),
        );
    }
}
