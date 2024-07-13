#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

impl Direction {
    #[inline(always)]
    pub fn opposite(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::UpLeft => Self::DownRight,
            Self::UpRight => Self::DownLeft,
            Self::DownLeft => Self::UpRight,
            Self::DownRight => Self::UpLeft,
        }
    }

    #[inline(always)]
    pub fn to_offset(self) -> (i16, i16) {
        match self {
            Self::Up => (0, 1),
            Self::Down => (0, -1),
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
            Self::UpLeft => (-1, 1),
            Self::UpRight => (1, 1),
            Self::DownLeft => (-1, -1),
            Self::DownRight => (1, -1),
        }
    }
}
