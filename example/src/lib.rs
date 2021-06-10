#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum CacheName {
    A,
    B,
    CommandAndControl,
    Invalid,
}

impl From<u8> for CacheName {
    fn from(x: u8) -> Self {
        match x {
            0   => Self::A,
            1   => Self::B,
            255 => Self::CommandAndControl,
            _   => Self::Invalid,
        }
    }
}

impl Into<u8> for CacheName {
    fn into(self) -> u8 {
        match self {
            Self::A => 0,
            Self::B => 1,
            Self::CommandAndControl => 255,
            Self::Invalid => 255
        }
    }
}

