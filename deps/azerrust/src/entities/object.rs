use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
};

use crate::{
    entities::{creature::CreatureRef, game_object::GameObjectRef, player::PlayerRef},
    map::MapRef,
    TypeId,
};

#[cxx::bridge]
pub mod ffi {
    extern "C++" {
        type Unit = crate::entities::unit::ffi::Unit;
        type Creature = crate::entities::creature::ffi::Creature;
        type Map = crate::map::ffi::Map;
        type GameObject = crate::entities::game_object::ffi::GameObject;
    }

    unsafe extern "C++" {
        include!("Object.h");
        type WorldObject;

        unsafe fn ToUnit(self: Pin<&mut Self>) -> *mut Unit;
        // This function is more safe since the conversion here is correct from &mut to *mut
        unsafe fn ToCreature(self: Pin<&mut Self>) -> *mut Creature;
        unsafe fn ToGameObject(self: Pin<&mut Self>) -> *mut GameObject;

        // TODO: Address this ASAP!!! This is very spooky but the C++ side requires that the Object reference is const but the
        // returned player reference is not. DO NOT USE THIS FUNCTION IF YOU CAN HELP IT
        unsafe fn GetMap(&self) -> *mut Map;

        fn GetZoneId(&self) -> u32;
        fn GetMapId(&self) -> u32;
        fn GetAreaId(&self) -> u32;
        fn GetPhaseMask(&self) -> u32;
        fn isType(&self, typeMask: u16) -> bool;
        fn GetEntry(&self) -> u32;

        //TODO: Another bad function
        unsafe fn FindNearestCreature(&self, entry: u32, range: f32, alive: bool) -> *mut Creature;

        //TODO: Another bad function
        unsafe fn FindNearestGameObject(
            &self,
            entry: u32,
            range: f32,
            only_spawned: bool,
        ) -> *mut GameObject;
    }
}

pub struct WorldObjectRef<'a>(pub Pin<&'a mut ffi::WorldObject>);

impl<'a> Deref for WorldObjectRef<'a> {
    type Target = Pin<&'a mut ffi::WorldObject>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for WorldObjectRef<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> WorldObjectRef<'a> {
    pub fn into_player(mut self) -> Result<PlayerRef<'a>, ()> {
        let unit = self.as_unit()?;
        // # Safety
        // C++ call ensures that the pointer returned here is nullable or a mutable reference to player
        let player = unsafe { unit.GetCharmerOrOwnerPlayerOrPlayerItself().as_mut() }.ok_or(())?;
        // # Safety
        // C++ call ensures that the value can be pinned and is not moved while Rust has the reference
        let player = unsafe { Pin::new_unchecked(player) };
        Ok(PlayerRef(player))
    }

    pub fn as_unit(&mut self) -> Result<Pin<&'a mut ffi::Unit>, ()> {
        // # Safety
        // C++ call ensures that the pointer returned here is nullable or a mutable reference to unit
        let unit = unsafe { self.0.as_mut().ToUnit().as_mut() }.ok_or(())?;

        // # Safety
        // C++ call ensures that the value can be pinned and is not moved while Rust has the reference
        let unit = unsafe { Pin::new_unchecked(unit) };
        Ok(unit)
    }

    //TODO: Make this an into but that requires without inheritance
    pub fn as_creature(&mut self) -> Option<CreatureRef<'a>> {
        // # Safety
        // C++ call ensures that the pointer returned here is nullable or a mutable reference to creature
        let creature = unsafe { self.0.as_mut().ToCreature().as_mut() }?;

        // # Safety
        // C++ call ensures that the value can be pinned and is not moved while Rust has the reference
        let creature = unsafe { Pin::new_unchecked(creature) };
        Some(CreatureRef(creature))
    }

    pub fn as_game_object(&mut self) -> Option<GameObjectRef<'a>> {
        // # Safety
        // C++ call ensures that the pointer returned here is nullable or a mutable reference to game object
        let game_object = unsafe { self.0.as_mut().ToGameObject().as_mut() }?;

        // # Safety
        // C++ call ensures that the value can be pinned and is not moved while Rust has the reference
        let game_object = unsafe { Pin::new_unchecked(game_object) };
        Some(GameObjectRef(game_object))
    }

    pub fn map(&mut self) -> Result<MapRef<'a>, ()> {
        // # Safety
        // C++ call ensures that the pointer returned here is nullable or a mutable reference to map
        let map = unsafe { self.0.GetMap().as_mut() }.ok_or(())?;
        // # Safety
        // C++ call ensures that the value can be pinned and is not moved while Rust has the reference
        let map = unsafe { Pin::new_unchecked(map) };
        Ok(MapRef(map))
    }

    pub fn find_nearest_creature(
        &mut self,
        entry: u32,
        range: f32,
        alive: bool,
    ) -> Option<CreatureRef<'a>> {
        Some(CreatureRef(unsafe {
            Pin::new_unchecked(self.FindNearestCreature(entry, range, alive).as_mut()?)
        }))
    }

    pub fn find_nearest_game_object(
        &mut self,
        entry: u32,
        range: f32,
        only_spawned: bool,
    ) -> Option<GameObjectRef<'a>> {
        Some(GameObjectRef(unsafe {
            Pin::new_unchecked(
                self.FindNearestGameObject(entry, range, only_spawned)
                    .as_mut()?,
            )
        }))
    }

    pub fn type_id(&self) -> Result<TypeId, ()> {
        let raw = crate::ffi::azerrust_worldobject_get_type_id(&self.0);
        TypeId::from_repr(raw).ok_or(())
    }
}
