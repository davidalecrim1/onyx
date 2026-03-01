use std::fmt;

/// All fallible operations in the Onyx workspace funnel through this type.
#[derive(Debug)]
pub enum OnyxError {
    Io(std::io::Error),
    TomlDeserialize(toml::de::Error),
    TomlSerialize(toml::ser::Error),
    NoHomeDir,
}

impl fmt::Display for OnyxError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "IO error: {error}"),
            Self::TomlDeserialize(error) => write!(formatter, "TOML parse error: {error}"),
            Self::TomlSerialize(error) => write!(formatter, "TOML serialize error: {error}"),
            Self::NoHomeDir => write!(formatter, "could not determine home directory"),
        }
    }
}

impl std::error::Error for OnyxError {}

impl From<std::io::Error> for OnyxError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<toml::de::Error> for OnyxError {
    fn from(error: toml::de::Error) -> Self {
        Self::TomlDeserialize(error)
    }
}

impl From<toml::ser::Error> for OnyxError {
    fn from(error: toml::ser::Error) -> Self {
        Self::TomlSerialize(error)
    }
}
