use eyre::EyreHandler;
use eyre::bail;
use facet::Facet;

pub enum Toggle {
    On,
    Off,
}

pub struct ToggleProxy(String);

impl TryFrom<ToggleProxy> for Toggle {
    type Error = eyre::Report;

    fn try_from(enabled: ToggleProxy) -> Result<Self, Self::Error> {
        match enabled.0.as_str() {
            "on" => Ok(Self::On),
            "off" => Ok(Self::Off),
            _ => bail!("invalid value"),
        }
    }
}

impl TryFrom<&Toggle> for ToggleProxy {
    type Error = std::convert::Infallible;

    fn try_from(enabled: &Toggle) -> Result<Self, Self::Error> {
        match enabled {
            Toggle::On => Ok(Self(String::from("on"))),
            Toggle::Off => Ok(Self(String::from("off"))),
        }
    }
}
