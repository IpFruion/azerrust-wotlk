use std::{ops::Deref, pin::Pin};

use strum::FromRepr;

#[cxx::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("GameObject.h");
        type GameObject;

        unsafe fn AI(&self) -> *mut GameObjectAI;
        fn GetSpawnId(&self) -> u32;
    }

    unsafe extern "C++" {
        include!("GameObjectAI.h");
        type GameObjectAI;

        fn GetData(&self, value: u32) -> u32;
    }

    unsafe extern "C++" {
        include!("azerrust_helpers.h");
        fn azerrust_game_object_go_state(game_object: &GameObject) -> u8;
    }
}

pub struct GameObjectRef<'a>(pub Pin<&'a mut ffi::GameObject>);

impl<'a> Deref for GameObjectRef<'a> {
    type Target = Pin<&'a mut ffi::GameObject>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> GameObjectRef<'a> {
    pub fn go_state(&self) -> Option<GOState> {
        GOState::from_repr(ffi::azerrust_game_object_go_state(&self.0))
    }

    pub fn ai(&mut self) -> Option<Pin<&'a mut ffi::GameObjectAI>> {
        Some(unsafe { Pin::new_unchecked(self.0.AI().as_mut()?) })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
#[repr(u8)]
pub enum GOState {
    Active = 0,
    Ready,
    Alternative,
}
