use std::{ops::Deref, pin::Pin};

use strum::FromRepr;

#[cxx::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("Map.h");
        type Map;

        fn GetSpawnMode(&self) -> u8;
    }
}

pub struct MapRef<'a>(pub Pin<&'a mut ffi::Map>);

impl<'a> Deref for MapRef<'a> {
    type Target = Pin<&'a mut ffi::Map>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> MapRef<'a> {
    /// From `Map.h`
    /// [[nodiscard]] Difficulty GetDifficulty() const { return Difficulty(GetSpawnMode()); }
    pub fn difficulty(&self) -> Result<Difficulty, ()> {
        Difficulty::from_repr(self.GetSpawnMode()).ok_or(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
#[repr(u8)]
pub enum Difficulty {
    Normal10 = 0,
    Normal25,
    Heroic10,
    Heroic25,
}

/// To match the C++ enum we can define these constants as aliases
#[allow(unused)]
impl Difficulty {
    pub const REGULAR: Difficulty = Difficulty::Normal10;
    pub const DUNGEON_NORMAL: Difficulty = Difficulty::Normal10;
    pub const DUNGEON_HEROIC: Difficulty = Difficulty::Normal25;
    pub const DUNGEON_EPIC: Difficulty = Difficulty::Heroic10;
}
