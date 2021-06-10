#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Get,
    MGet,
    Keys,
    Exists,
    MExists,

    Set,
    MSet,
    Del,
    MDel,

    Save,

    Pong,
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

impl Into<u8> for Command {
    fn into(self) -> u8 {
        match self {
            Self::Get     => 0,
            Self::MGet    => 1,
            Self::Keys    => 2,
            Self::Exists  => 3,
            Self::MExists => 4,

            Self::Set     => 5,
            Self::MSet    => 6,
            Self::Del     => 7,
            Self::MDel    => 8,

            Self::Save    => 9,

            Self::Ping    => 254,
            Self::Pong    => 255,
        }
    }
}

