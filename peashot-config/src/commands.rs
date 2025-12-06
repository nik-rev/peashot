use facet::Facet;
use facet_kdl as kdl;

use crate::{
    key_mods::{KeyMods, KeyModsProxy},
    key_seq::{KeySequence, KeySequenceProxy},
};

#[derive(Facet, PartialEq, Debug)]
#[repr(C)]
#[facet(rename_all = "kebab-case")]
pub enum Place {
    /// Center
    Center,
    /// Center on the x-axis
    XCenter,
    /// Center on the y-axis
    YCenter,
    /// Top-left corner
    TopLeft,
    /// Bottom-left corner
    BottomLeft,
    /// Bottom-right corner
    BottomRight,
    /// Top-right corner
    TopRight,
    /// Left side
    Left,
    /// Right side
    Right,
    /// Top side
    Top,
    /// Bottom side
    Bottom,
}

#[derive(Facet, PartialEq, Debug)]
#[repr(C)]
#[facet(rename_all = "kebab-case")]
pub enum Direction {
    /// Above
    Up,
    /// Below
    Down,
    /// To the left
    Left,
    /// To the right
    Right,
}

#[derive(Facet, Debug, PartialEq)]
#[repr(C)]
#[facet(rename_all = "kebab-case")]
pub enum Command {
    /// Copy image to the clipboard
    UploadScreenshot {
        // #[facet(rename = "key", kdl::property, proxy = KeySequenceProxy)]
        // key: KeySequence,
        // #[facet(rename = "mod", kdl::property, proxy = KeyModsProxy)]
        // mods: KeyMods,
    },
    /// Save image to a file
    CopyToClipboard {
        // #[facet(rename = "key", kdl::property, proxy = KeySequenceProxy)]
        // key: KeySequence,
        // #[facet(rename = "mod", kdl::property, proxy = KeyModsProxy)]
        // mods: KeyMods,
    },
    /// Upload image to the internet
    SaveScreenshot {
        // #[facet(rename = "key", kdl::property, proxy = KeySequenceProxy)]
        // key: KeySequence,
        // #[facet(rename = "mod", kdl::property, proxy = KeyModsProxy)]
        // mods: KeyMods,
    },
    /// Do nothing
    NoOp {
        // #[facet(rename = "key", kdl::property, proxy = KeySequenceProxy)]
        // key: KeySequence,
        // #[facet(rename = "mod", kdl::property, proxy = KeyModsProxy)]
        // mods: KeyMods,
    },
    /// Exit the application
    Exit {
        // #[facet(rename = "key", kdl::property, proxy = KeySequenceProxy)]
        // key: KeySequence,
        // #[facet(rename = "mod", kdl::property, proxy = KeyModsProxy)]
        // mods: KeyMods,
    },
    /// Toggle the overlay showing various information for debugging
    ToggleDebugOverlay {
        // #[facet(rename = "key", kdl::property, proxy = KeySequenceProxy)]
        // key: KeySequence,
        // #[facet(rename = "mod", kdl::property, proxy = KeyModsProxy)]
        // mods: KeyMods,
    },
    /// Open the keybindings cheatsheet
    OpenKeybindingsCheatsheet {
        // #[facet(rename = "key", kdl::property, proxy = KeySequenceProxy)]
        // key: KeySequence,
        // #[facet(rename = "mod", kdl::property, proxy = KeyModsProxy)]
        // mods: KeyMods,
    },
    /// Open a grid of letters to pick the top left corner in 3 keystrokes
    PickTopLeftCorner {
        // #[facet(rename = "key", kdl::property, proxy = KeySequenceProxy)]
        // key: KeySequence,
        // #[facet(rename = "mod", kdl::property, proxy = KeyModsProxy)]
        // mods: KeyMods,
    },
    /// Open a grid of letters to pick the bottom right corner in 3 keystrokes
    PickBottomRightCorner {
        // #[facet(rename = "key", kdl::property, proxy = KeySequenceProxy)]
        // key: KeySequence,
        // #[facet(rename = "mod", kdl::property, proxy = KeyModsProxy)]
        // mods: KeyMods,
    },
    /// Set the width to whatever number is currently pressed
    SetWidth {
        // #[facet(rename = "key", kdl::property, proxy = KeySequenceProxy)]
        // key: KeySequence,
        // #[facet(rename = "mod", kdl::property, proxy = KeyModsProxy)]
        // mods: KeyMods,
    },
    /// Set the height to whatever number is currently pressed
    SetHeight {
        // #[facet(rename = "key", kdl::property, proxy = KeySequenceProxy)]
        // key: KeySequence,
        // #[facet(rename = "mod", kdl::property, proxy = KeyModsProxy)]
        // mods: KeyMods,
    },
    // /// Set selection to encompass the entire screen
    // SelectRegion {
    //     #[ferrishot_knus(str)]
    //     selection: LazyRectangle,
    // },
    /// Remove the selection
    ClearSelection {
        // #[facet(rename = "key", kdl::property, proxy = KeySequenceProxy)]
        // key: KeySequence,
        // #[facet(rename = "mod", kdl::property, proxy = KeyModsProxy)]
        // mods: KeyMods,
    },
    /// Shift the selection in the given direction by pixels
    Move {
        #[facet(kdl::argument)]
        direction: Direction,
        #[facet(kdl::argument, default = u32::MAX)]
        amount: u32,
        // #[facet(rename = "key", kdl::property, proxy = KeySequenceProxy)]
        // key: KeySequence,
        // #[facet(rename = "mod", kdl::property, proxy = KeyModsProxy)]
        // mods: KeyMods,
    },
    /// Increase the size of the selection in the given direction by pixels
    Extend {
        #[facet(kdl::argument)]
        direction: Direction,
        #[facet(kdl::argument, default = u32::MAX)]
        amount: u32,
        // #[facet(rename = "key", kdl::property, proxy = KeySequenceProxy)]
        // key: KeySequence,
        // #[facet(rename = "mod", kdl::property, proxy = KeyModsProxy)]
        // mods: KeyMods,
    },
    /// Decrease the size of the selection in the given direction by pixels
    Shrink {
        #[facet(kdl::argument)]
        direction: Direction,
        #[facet(kdl::argument, default = u32::MAX)]
        amount: u32,
        // #[facet(rename = "key", kdl::property, proxy = KeySequenceProxy)]
        // key: KeySequence,
        // #[facet(rename = "mod", kdl::property, proxy = KeyModsProxy)]
        // mods: KeyMods,
    },
    /// Move rectangle to a place
    Goto {
        #[facet(kdl::argument)]
        place: Place,
        // #[facet(rename = "key", kdl::property, proxy = KeySequenceProxy)]
        // key: KeySequence,
        // #[facet(rename = "mod", kdl::property, proxy = KeyModsProxy)]
        // mods: KeyMods,
    },
}
