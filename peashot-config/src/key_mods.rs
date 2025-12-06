use core::fmt;
use facet::Facet;
use iced::keyboard::Modifiers;

/// Modifier keys
#[derive(Debug, Default, Clone, Hash, Eq, PartialEq, Facet)]
pub struct KeyMods(#[facet(opaque)] pub Modifiers);

crate::create_string_proxy!(KeyMods <=> KeyModsProxy);

impl std::str::FromStr for KeyMods {
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

impl fmt::Display for KeyMods {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.contains(Modifiers::CTRL) {
            f.write_str("ctrl")?;
        }
        if self.0.contains(Modifiers::ALT) {
            f.write_str("alt")?;
        }
        if self.0.contains(Modifiers::LOGO) {
            cfg_if::cfg_if! {
                if #[cfg(target_os = "windows")] {
                    f.write_str("windows")?;
                } else if #[cfg(target_os = "macos")] {
                    f.write_str("command")?;
                } else {
                    f.write_str("super")?;
                }
            }
        }
        if self.0.contains(Modifiers::SHIFT) {
            f.write_str("shift")?;
        }
        Ok(())
    }
}
