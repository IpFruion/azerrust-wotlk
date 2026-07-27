use crate::{ffi::azerrust_achievement_store_lookup, game_event_mgr, world_state};
use enum_dispatch::enum_dispatch;

#[enum_dispatch(GlobalConditionTrait)]
pub enum GlobalConditionKind {
    ActiveEvent(GlobalConditionActiveEvent),
    WorldState(GlobalConditionWorldState),
    WorldScript(GlobalConditionWorldScript),
    RealmAchievement(GlobalConditionRealmAchievement),
}

#[enum_dispatch]
pub trait GlobalConditionTrait {
    fn meets(&self) -> bool;
}

pub struct GlobalConditionActiveEvent {
    event_id: u32,
}

impl GlobalConditionTrait for GlobalConditionActiveEvent {
    fn meets(&self) -> bool {
        game_event_mgr().IsActiveEvent(self.event_id as u16)
    }
}

impl TryFrom<(u32, u32, u32)> for GlobalConditionActiveEvent {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(GlobalConditionActiveEvent { event_id: value.0 })
    }
}

pub struct GlobalConditionWorldState {
    index: u32,
    value: u32,
}

impl GlobalConditionTrait for GlobalConditionWorldState {
    fn meets(&self) -> bool {
        let ws = world_state();
        ws.getWorldState(self.index) == u64::from(self.value)
    }
}

impl TryFrom<(u32, u32, u32)> for GlobalConditionWorldState {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(GlobalConditionWorldState {
            index: value.0,
            value: value.1,
        })
    }
}

pub struct GlobalConditionWorldScript {
    condition_id: u32,
    state: u32,
}

impl GlobalConditionTrait for GlobalConditionWorldScript {
    fn meets(&self) -> bool {
        let ws = world_state();
        ws.IsConditionFulfilled(self.condition_id, self.state)
    }
}

impl TryFrom<(u32, u32, u32)> for GlobalConditionWorldScript {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(GlobalConditionWorldScript {
            condition_id: value.0,
            state: value.1,
        })
    }
}

pub struct GlobalConditionRealmAchievement {
    achievement_id: u32,
}

impl GlobalConditionTrait for GlobalConditionRealmAchievement {
    fn meets(&self) -> bool {
        let achievement =
            unsafe { azerrust_achievement_store_lookup(self.achievement_id).as_ref() };
        achievement.is_some_and(|a| {
            unsafe { crate::ffi::azerrust_achievement_mgr().as_ref() }
                .is_some_and(|mgr| unsafe { mgr.IsRealmCompleted(a) })
        })
    }
}

impl TryFrom<(u32, u32, u32)> for GlobalConditionRealmAchievement {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(GlobalConditionRealmAchievement {
            achievement_id: value.0,
        })
    }
}
