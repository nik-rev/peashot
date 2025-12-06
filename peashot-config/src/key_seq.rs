use core::fmt;

use eyre::bail;
use facet::Facet;
use iced::{advanced::graphics::core::SmolStr, keyboard::key::Key as IcedKey};

/// A sequence of 2 keys. If there are 2 keys like so:
/// - (T, None)
/// - (T, Some(X))
///
/// The 2nd key will never be triggered.
/// We will first search the `HashMap` of keys for the first key.
/// If it does not exist, search for the 2nd key.
#[derive(Debug, Hash, PartialEq, PartialOrd, Ord, Eq, Clone, Facet)]
pub struct KeySequence(#[facet(opaque)] pub (IcedKey, Option<IcedKey>));

crate::create_string_proxy!(KeySequence <=> KeySequenceProxy);

impl fmt::Display for KeySequence {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl std::str::FromStr for KeySequence {
    type Err = eyre::Report;

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
                            named_key_buf.parse::<crate::key_named::Named>()?.to_iced(),
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
            bail!("Expected at least 1 key.");
        };
        let second_key = keys.next();
        if keys.next().is_some() {
            // because we only store a single previous key, we can't handle keybindings
            // with more than 1 key. Since this is a screenshot app and not something like a
            // text editor, I don't believe there is much utility in allowing 3 keys in a row or more.
            //
            // This greatly simplifies the code, as we don't have to be generic.
            bail!("At the moment, only up to 2 keys in a sequence are supported.");
        }
        Ok(Self((first_key, second_key)))
    }
}

#[cfg(test)]
mod test {
    use iced::keyboard::key;

    use super::*;
    // use pretty_assertions::assert_eq;

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
