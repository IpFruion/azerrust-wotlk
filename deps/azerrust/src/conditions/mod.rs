pub mod comparison;
pub mod global;
pub mod kind;
pub mod multi_target;
pub mod player;
pub mod unit;
pub mod world_object;

use std::{num::NonZeroU8, pin::Pin};

use crate::{
    conditions::{
        global::GlobalConditionTrait, kind::ConditionKind, multi_target::MultiTargetConditionTrait,
        player::PlayerConditionTrait, unit::UnitConditionTrait,
        world_object::WorldObjectConditionTrait,
    },
    entities::object::WorldObjectRef,
};

#[cxx::bridge]
mod ffi {
    #[namespace = "conditions"]
    struct Condition {
        cond_type: u8,
        val1: u32,
        val2: u32,
        val3: u32,
        negative: bool,
        target: u8,
    }

    extern "C++" {
        include!("Object.h");
        type WorldObject = crate::entities::object::ffi::WorldObject;
    }

    unsafe extern "C++" {
        include!("azerrust_helpers.h");
        type ConditionSourceInfo;

        fn azerrust_source_info_get_target(
            info: Pin<&mut ConditionSourceInfo>,
            index: u8,
        ) -> *mut WorldObject;
    }

    #[namespace = "conditions"]
    extern "Rust" {
        unsafe fn meets(sourceInfo: Pin<&mut ConditionSourceInfo>, cond: Condition) -> i8;
    }
}

fn meets(source_info: Pin<&mut ffi::ConditionSourceInfo>, cond: ffi::Condition) -> i8 {
    fn try_meets(source_info: ConditionSourceInfoRef, cond: ffi::Condition) -> Result<bool, ()> {
        let condition: Condition = cond.try_into()?;
        condition.meets(source_info)
    }

    try_meets(ConditionSourceInfoRef(source_info), cond)
        .map(Into::into)
        .unwrap_or(-1)
}

pub struct ConditionSourceInfoRef<'a>(Pin<&'a mut ffi::ConditionSourceInfo>);

impl<'a> ConditionSourceInfoRef<'a> {
    fn get_target(&mut self, idx: u8) -> Option<WorldObjectRef<'a>> {
        Some(WorldObjectRef(unsafe {
            Pin::new_unchecked(ffi::azerrust_source_info_get_target(self.0.as_mut(), idx).as_mut()?)
        }))
    }
}

pub struct Condition {
    target: u8,
    kind: Option<ConditionKind>,
    negative: bool,
}

impl Condition {
    pub fn meets(&self, mut source_info: ConditionSourceInfoRef) -> Result<bool, ()> {
        let res = match self.kind.as_ref() {
            None => Ok(true),
            Some(ConditionKind::Unit(unit_cond)) => {
                let mut object = source_info.get_target(self.target).ok_or(())?;
                let unit = object.as_unit()?;
                Ok(unit_cond.meets(unit.as_ref()))
            }
            Some(ConditionKind::WorldObject(wo_cond)) => {
                let object = source_info.get_target(self.target).ok_or(())?;
                wo_cond.meets(object)
            }
            Some(ConditionKind::Player(player_cond)) => {
                let object = source_info.get_target(self.target).ok_or(())?;
                let player = object.into_player()?;
                player_cond.meets(player)
            }
            Some(ConditionKind::Global(global_cond)) => Ok(global_cond.meets()),
            Some(ConditionKind::MultiTarget(mt_cond)) => {
                let object = source_info.get_target(self.target).ok_or(())?;
                let target = source_info.get_target(mt_cond.target()).ok_or(())?;
                mt_cond.meets(object, target)
            }
        }?;

        Ok(if self.negative { !res } else { res })
    }
}

impl TryFrom<ffi::Condition> for Condition {
    type Error = ();

    fn try_from(value: ffi::Condition) -> Result<Self, Self::Error> {
        let negative = value.negative;
        let target = value.target;
        let kind = NonZeroU8::new(value.cond_type)
            .map(|cond| {
                let values = (value.val1, value.val2, value.val3);
                ConditionKind::try_from((cond, values))
            })
            .transpose()?;
        Ok(Condition {
            kind,
            negative,
            target,
        })
    }
}
