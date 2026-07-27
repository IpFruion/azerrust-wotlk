use enum_dispatch::enum_dispatch;

use crate::{
    UnitFlags,
    entities::{game_object::GOState, object::WorldObjectRef},
    map::Difficulty,
};

#[enum_dispatch(WorldObjectConditionTrait)]
pub enum WorldObjectConditionKind {
    ZoneId(WorldObjectConditionZoneId),
    MapId(WorldObjectConditionMapId),
    AreaId(WorldObjectConditionAreaId),
    PhaseMask(WorldObjectConditionPhaseMask),
    TypeMask(WorldObjectConditionTypeMask),
    SpawnMask(WorldObjectConditionSpawnMask),
    DifficultyId(WorldObjectConditionDifficultyId),
    CreatureType(WorldObjectConditionCreatureType),
    NearCreature(WorldObjectConditionNearCreature),
    NearGameObject(WorldObjectConditionNearGameObject),
    ObjectEntryGuid(WorldObjectConditionObjectEntryGuid),
    InstanceInfo(WorldObjectConditionInstanceInfo),
    AiData(WorldObjectConditionAiData),
}

#[enum_dispatch]
pub trait WorldObjectConditionTrait {
    fn meets(&self, object: WorldObjectRef) -> Result<bool, ()>;
}

pub struct WorldObjectConditionZoneId {
    zone_id: u32,
}

impl WorldObjectConditionTrait for WorldObjectConditionZoneId {
    fn meets(&self, object: WorldObjectRef) -> Result<bool, ()> {
        Ok(object.GetZoneId() == self.zone_id)
    }
}

impl TryFrom<(u32, u32, u32)> for WorldObjectConditionZoneId {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(WorldObjectConditionZoneId { zone_id: value.0 })
    }
}

pub struct WorldObjectConditionMapId {
    map_id: u32,
}

impl WorldObjectConditionTrait for WorldObjectConditionMapId {
    fn meets(&self, object: WorldObjectRef) -> Result<bool, ()> {
        Ok(object.GetMapId() == self.map_id)
    }
}

impl TryFrom<(u32, u32, u32)> for WorldObjectConditionMapId {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(WorldObjectConditionMapId { map_id: value.0 })
    }
}

pub struct WorldObjectConditionAreaId {
    area_id: u32,
}

impl WorldObjectConditionTrait for WorldObjectConditionAreaId {
    fn meets(&self, object: WorldObjectRef) -> Result<bool, ()> {
        Ok(object.GetAreaId() == self.area_id)
    }
}

impl TryFrom<(u32, u32, u32)> for WorldObjectConditionAreaId {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(WorldObjectConditionAreaId { area_id: value.0 })
    }
}

pub struct WorldObjectConditionPhaseMask {
    phase_mask: u32,
}

impl WorldObjectConditionTrait for WorldObjectConditionPhaseMask {
    fn meets(&self, object: WorldObjectRef) -> Result<bool, ()> {
        Ok(object.GetPhaseMask() & self.phase_mask != 0)
    }
}

impl TryFrom<(u32, u32, u32)> for WorldObjectConditionPhaseMask {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(WorldObjectConditionPhaseMask {
            phase_mask: value.0,
        })
    }
}

pub struct WorldObjectConditionTypeMask {
    type_mask: u16,
}

impl WorldObjectConditionTrait for WorldObjectConditionTypeMask {
    fn meets(&self, object: WorldObjectRef) -> Result<bool, ()> {
        Ok(object.isType(self.type_mask))
    }
}

impl TryFrom<(u32, u32, u32)> for WorldObjectConditionTypeMask {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(WorldObjectConditionTypeMask {
            type_mask: value.0 as u16,
        })
    }
}

pub struct WorldObjectConditionSpawnMask {
    spawn_mask: u32,
}

impl WorldObjectConditionTrait for WorldObjectConditionSpawnMask {
    fn meets(&self, mut object: WorldObjectRef) -> Result<bool, ()> {
        let map = object.map()?;
        Ok((1u32 << u32::from(map.GetSpawnMode())) & self.spawn_mask != 0)
    }
}

impl TryFrom<(u32, u32, u32)> for WorldObjectConditionSpawnMask {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(WorldObjectConditionSpawnMask {
            spawn_mask: value.0,
        })
    }
}

pub struct WorldObjectConditionDifficultyId {
    difficulty: Difficulty,
}

impl WorldObjectConditionTrait for WorldObjectConditionDifficultyId {
    fn meets(&self, mut object: WorldObjectRef) -> Result<bool, ()> {
        let map = object.map()?;
        let current = map.difficulty()?;
        Ok(current == self.difficulty)
    }
}

impl TryFrom<(u32, u32, u32)> for WorldObjectConditionDifficultyId {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(WorldObjectConditionDifficultyId {
            difficulty: Difficulty::from_repr(value.0 as u8).ok_or(())?,
        })
    }
}

pub struct WorldObjectConditionCreatureType {
    creature_type: u32,
}

impl WorldObjectConditionTrait for WorldObjectConditionCreatureType {
    fn meets(&self, mut object: WorldObjectRef) -> Result<bool, ()> {
        let Some(creature) = object.as_creature() else {
            return Ok(false);
        };
        let template = creature.creature_template()?;
        Ok(crate::ffi::azerrust_creaturetemplate_get_type(template) == self.creature_type)
    }
}

impl TryFrom<(u32, u32, u32)> for WorldObjectConditionCreatureType {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(WorldObjectConditionCreatureType {
            creature_type: value.0,
        })
    }
}

pub struct WorldObjectConditionNearCreature {
    entry: u32,
    distance: f32,
    alive: bool,
}

impl WorldObjectConditionTrait for WorldObjectConditionNearCreature {
    fn meets(&self, mut object: WorldObjectRef) -> Result<bool, ()> {
        Ok(object
            .find_nearest_creature(self.entry, self.distance, self.alive)
            .is_some())
    }
}

impl TryFrom<(u32, u32, u32)> for WorldObjectConditionNearCreature {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(WorldObjectConditionNearCreature {
            entry: value.0,
            distance: f32::from_bits(value.1),
            alive: value.2 == 0,
        })
    }
}

pub struct WorldObjectConditionNearGameObject {
    entry: u32,
    distance: f32,
    go_state: u32,
}

impl WorldObjectConditionTrait for WorldObjectConditionNearGameObject {
    fn meets(&self, mut object: WorldObjectRef) -> Result<bool, ()> {
        let Some(obj) = object.find_nearest_game_object(self.entry, self.distance, false) else {
            return Ok(false);
        };
        Ok(match self.go_state {
            0 => true,
            1 => obj.go_state().ok_or(())? == GOState::Ready,
            _ => obj.go_state().ok_or(())? != GOState::Ready,
        })
    }
}

impl TryFrom<(u32, u32, u32)> for WorldObjectConditionNearGameObject {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(WorldObjectConditionNearGameObject {
            entry: value.0,
            distance: f32::from_bits(value.1),
            go_state: value.2,
        })
    }
}

use crate::TypeId;

pub struct WorldObjectConditionObjectEntryGuid {
    type_id: TypeId,
    entry: u32,
    value3: u32,
}

impl WorldObjectConditionTrait for WorldObjectConditionObjectEntryGuid {
    fn meets(&self, mut object: WorldObjectRef) -> Result<bool, ()> {
        // value3 == 1: skip non-attackable/not-selectable units
        if self.value3 == 1
            && let Ok(unit) = object.as_unit()
            && crate::ffi::azerrust_unit_has_unit_flag(
                &unit,
                (UnitFlags::IMMUNE_TO_PC | UnitFlags::NOT_SELECTABLE).bits(),
            )
        {
            return Ok(false);
        }
        let tid = object.type_id()?;
        if tid != self.type_id {
            return Ok(false);
        }
        if self.entry != 0 && object.GetEntry() != self.entry {
            return Ok(false);
        }
        if self.value3 > 1 {
            if tid == TypeId::Unit {
                let creature = object.as_creature().ok_or(())?;
                return Ok(creature.GetSpawnId() == self.value3);
            }
            if tid == TypeId::GameObject {
                let game_object = object.as_game_object().ok_or(())?;
                return Ok(game_object.GetSpawnId() == self.value3);
            }
            return Ok(false);
        }
        Ok(true)
    }
}

impl TryFrom<(u32, u32, u32)> for WorldObjectConditionObjectEntryGuid {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(WorldObjectConditionObjectEntryGuid {
            type_id: TypeId::from_repr(value.0).ok_or(())?,
            entry: value.1,
            value3: value.2,
        })
    }
}

pub struct WorldObjectConditionInstanceInfo {
    data_id: u32,
    data: u32,
    info_type: crate::InstanceInfo,
}

impl WorldObjectConditionTrait for WorldObjectConditionInstanceInfo {
    fn meets(&self, mut object: WorldObjectRef) -> Result<bool, ()> {
        let map = object.map()?;
        if !map.IsDungeon() {
            return Ok(false);
        }
        let script = unsafe { crate::ffi::azerrust_map_get_instance_script(&map).as_ref() };
        let Some(script) = script else {
            return Ok(false);
        };
        Ok(match self.info_type {
            crate::InstanceInfo::Data => script.GetData(self.data_id) == self.data,
            crate::InstanceInfo::GuidData => crate::ffi::azerrust_instance_script_check_guid_data(
                script,
                self.data_id,
                self.data,
            ),
            crate::InstanceInfo::BossState => {
                crate::ffi::azerrust_instance_script_get_boss_state(script, self.data_id)
                    == self.data
            }
            crate::InstanceInfo::Data64 => script.GetData64(self.data_id) == self.data as u64,
        })
    }
}

impl TryFrom<(u32, u32, u32)> for WorldObjectConditionInstanceInfo {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(WorldObjectConditionInstanceInfo {
            data_id: value.0,
            data: value.1,
            info_type: crate::InstanceInfo::from_repr(value.2).ok_or(())?,
        })
    }
}

pub struct WorldObjectConditionAiData {
    data_id: u32,
    expected_value: u32,
}

impl WorldObjectConditionTrait for WorldObjectConditionAiData {
    fn meets(&self, mut object: WorldObjectRef) -> Result<bool, ()> {
        if let Some(mut creature) = object.as_creature() {
            return Ok(creature
                .ai()
                .is_some_and(|ai| ai.GetData(self.data_id) == self.expected_value));
        }
        if let Some(mut game_object) = object.as_game_object() {
            return Ok(game_object
                .ai()
                .is_some_and(|ai| ai.GetData(self.data_id) == self.expected_value));
        }

        Ok(false)
    }
}

impl TryFrom<(u32, u32, u32)> for WorldObjectConditionAiData {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(WorldObjectConditionAiData {
            data_id: value.0,
            expected_value: value.1,
        })
    }
}
