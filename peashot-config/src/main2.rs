#![allow(clippy::result_large_err)]
#![allow(clippy::infallible_try_from)]

mod color;
mod commands;
mod key_mods;
mod key_named;
mod key_seq;
mod toggle;

use std::collections::HashMap;

use facet::Facet;
use facet_kdl as kdl;

use facet_pretty::FacetPretty;
use subdef::subdef;

use crate::toggle::Toggle;

#[macro_export]
macro_rules! create_string_proxy {
    ($ty:ident <=> $proxy:ident) => {
        #[derive(facet::Facet, Clone)]
        #[facet(transparent)]
        pub(crate) struct $proxy(String);

        impl TryFrom<$proxy> for $ty {
            type Error = <$ty as std::str::FromStr>::Err;

            fn try_from(proxy: $proxy) -> Result<Self, Self::Error> {
                <$ty as std::str::FromStr>::from_str(&proxy.0)
            }
        }

        impl TryFrom<&$ty> for $proxy {
            type Error = std::convert::Infallible;

            fn try_from(ty: &$ty) -> Result<Self, Self::Error> {
                Ok($proxy(ty.to_string()))
            }
        }
    };
}

#[subdef(
    derive(Facet, Debug, PartialEq),
    facet(rename_all = "kebab-case", deny_unknown_fields)
)]
struct Config {
    // #[facet(kdl::child, proxy = toggle::ToggleProxy)]
    // size_indicator: Toggle,
    // #[facet(kdl::child, proxy = toggle::ToggleProxy)]
    // selection_icons: Toggle,
    #[facet(kdl::child)]
    theme: [_; {
        struct Theme {
            #[facet(kdl::child)]
            palette: [_; {
                struct Palette(#[facet(kdl::children)] HashMap<color::PaletteId, color::KdlColor>);
            }],
            #[facet(kdl::children)]
            colors: HashMap<color::ColorId, color::KdlColor>,
        }
    }],
    #[facet(kdl::child)]
    keymap: [_; {
        struct KeyMap(#[facet(kdl::children)] Vec<crate::commands::Command>);
    }],
}

fn main() -> miette::Result<()> {
    let config = facet_kdl::from_str::<Config>(include_str!("config.kdl"))?;

    println!("{}", config.pretty());

    Ok(())
}
