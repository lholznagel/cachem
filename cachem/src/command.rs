//! List of all valid commands and a parser from and to u8.

/// Contains all valid commands
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// Gets a single item
    Get,
    /// Gets an array of items
    MGet,
    /// Gets a list of all keys for a cache
    Keys,
    /// Checks if a key exists
    Exists,
    /// Checks if an array of keys exist
    MExists,

    /// Sets a value at the given index
    Set,
    /// Sets a list of values to the given ids
    MSet,
    /// Deletes a single item
    Del,
    /// Deletes an array of items
    MDel,

    /// Saves the current cache to disk
    Save,

    /// Pong from the server
    Pong,
    /// Pings the server
    Ping,
}

impl From<u8> for Command {
    fn from(x: u8) -> Self {
        match x {
            0   => Self::Get,
            1   => Self::MGet,
            2   => Self::Keys,
            3   => Self::Exists,
            4   => Self::MExists,

            5   => Self::Set,
            6   => Self::MSet,
            7   => Self::Del,
            8   => Self::MDel,

            9   => Self::Save,

            254 => Self::Ping,
            _   => Self::Pong,
        }
    }
}

impl From<Command> for u8 {
    fn from(c: Command) -> u8 {
        match c {
            Command::Get     => 0,
            Command::MGet    => 1,
            Command::Keys    => 2,
            Command::Exists  => 3,
            Command::MExists => 4,

            Command::Set     => 5,
            Command::MSet    => 6,
            Command::Del     => 7,
            Command::MDel    => 8,

            Command::Save    => 9,

            Command::Ping    => 254,
            Command::Pong    => 255,
        }
    }
}
