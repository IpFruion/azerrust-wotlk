mod conditions;
mod entities;
mod error;
mod map;
mod quests;
mod reputation;

use std::pin::Pin;

use bitflags::bitflags;
use strum::FromRepr;

use crate::ffi::{azerrust_game_event_mgr, azerrust_world_state};

#[cxx::bridge]
mod ffi {
    extern "C++" {
        type CreatureTemplate = crate::entities::creature::ffi::CreatureTemplate;
        type Map = crate::map::ffi::Map;
        type Unit = crate::entities::unit::ffi::Unit;
        type Pet = crate::entities::pet::ffi::Pet;
        type Player = crate::entities::player::ffi::Player;
        type Quest = crate::quests::ffi::Quest;
        type WorldObject = crate::entities::object::ffi::WorldObject;
        type Creature = crate::entities::creature::ffi::Creature;
    }

    unsafe extern "C++" {
        include!("WorldState.h");
        type WorldState;

        fn getWorldState(&self, index: u32) -> u64;
        fn IsConditionFulfilled(&self, conditionId: u32, state: u32) -> bool;
    }

    unsafe extern "C++" {
        include!("GameEventMgr.h");
        type GameEventMgr;

        fn IsActiveEvent(self: Pin<&mut Self>, eventId: u16) -> bool;
    }

    unsafe extern "C++" {
        include!("ObjectMgr.h");
        type ObjectMgr;

        unsafe fn GetQuestTemplate(&self, questId: u32) -> *const Quest;
    }

    unsafe extern "C++" {
        include!("DBCStores.h");
        type FactionEntry;
    }

    unsafe extern "C++" {
        include!("AchievementMgr.h");
        type AchievementEntry;
        type AchievementGlobalMgr;

        unsafe fn IsRealmCompleted(
            self: &AchievementGlobalMgr,
            achievement: *const AchievementEntry,
        ) -> bool;
    }

    unsafe extern "C++" {
        include!("azerrust_helpers.h");
        fn azerrust_unit_has_aura_effect(unit: &Unit, spellId: u32, effIndex: u8) -> bool;
        fn azerrust_unit_has_aura_type(unit: &Unit, auraType: u32) -> bool;
        unsafe fn azerrust_world_state() -> *mut WorldState;
        unsafe fn azerrust_game_event_mgr() -> *mut GameEventMgr;
        fn azerrust_player_get_quest_status(player: &Player, questId: u32) -> u8;
        fn azerrust_player_get_team_id(player: &Player) -> u8;
        fn azerrust_player_queued_random_dungeon(
            player: &Player,
            checkDifficulty: u32,
            difficulty: u32,
        ) -> bool;
        fn azerrust_worldobject_get_type_id(obj: &WorldObject) -> u32;
        fn azerrust_unit_has_unit_flag(unit: &Unit, flag: u32) -> bool;
        fn azerrust_unit_has_owner(unit: &Unit, owner: &Unit) -> bool;
        fn azerrust_unit_has_creator(unit: &Unit, creator: &Unit) -> bool;
        fn azerrust_worldobject_check_instance_info(
            obj: &WorldObject,
            val1: u32,
            val2: u32,
            val3: u32,
        ) -> bool;
        fn azerrust_worldobject_check_reaction(obj: &WorldObject, target: &WorldObject) -> i8;
        fn azerrust_worldobject_get_distance(obj: &WorldObject, target: &WorldObject) -> f32;
        unsafe fn azerrust_achievement_mgr() -> *mut AchievementGlobalMgr;
        unsafe fn azerrust_achievement_store_lookup(id: u32) -> *const AchievementEntry;
        fn azerrust_pet_get_pet_type(pet: &Pet) -> u8;
        unsafe fn azerrust_faction_store_lookup(id: u32) -> *const FactionEntry;
        unsafe fn azerrust_player_reputation_rank(
            player: &Player,
            faction: *const FactionEntry,
        ) -> u8;
        fn azerrust_creaturetemplate_get_type(ct: &CreatureTemplate) -> u32;
        unsafe fn azerrust_object_mgr() -> *mut ObjectMgr;
    }
}

fn world_state() -> Pin<&'static mut ffi::WorldState> {
    unsafe {
        Pin::new_unchecked(
            azerrust_world_state()
                .as_mut()
                .expect("Static reference exists as is convertable"),
        )
    }
}

fn game_event_mgr() -> Pin<&'static mut ffi::GameEventMgr> {
    unsafe {
        Pin::new_unchecked(
            azerrust_game_event_mgr()
                .as_mut()
                .expect("Static reference exists as is convertable"),
        )
    }
}

// fn object_mgr() -> &'static ffi::ObjectMgr {
//     unsafe { crate::ffi::azerrust_object_mgr().as_ref() }
//         .expect("Static reference exists as is convertable")
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
#[repr(u8)]
pub enum TeamId {
    Alliance = 0,
    Horde,
    Neutral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
#[repr(u32)]
pub enum Team {
    Horde = 67,
    Alliance = 469,
    Other = 0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromRepr)]
#[repr(u32)]
pub enum DrunkenState {
    Sober = 0,
    Tipsy,
    Drunk,
    Smashed,
}

impl DrunkenState {
    pub fn from_value(value: u8) -> Self {
        match value {
            90.. => DrunkenState::Smashed,
            50.. => DrunkenState::Drunk,
            1.. => DrunkenState::Tipsy,
            0 => DrunkenState::Sober,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
#[repr(u32)]
pub enum Relation {
    Self_ = 0,
    InParty = 1,
    InRaidOrParty = 2,
    OwnedBy = 3,
    PassengerOf = 4,
    CreatedBy = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
#[repr(u32)]
pub enum TypeId {
    Object = 0,
    Item = 1,
    Container = 2,
    Unit = 3,
    Player = 4,
    GameObject = 5,
    DynamicObject = 6,
    Corpse = 7,
}

bitflags! {
    pub struct UnitFlags: u32 {
        /// set only when unit movement is controlled by server
        const SERVER_CONTROLLED = 0x00000001;
        /// not attackable
        const NON_ATTACKABLE = 0x00000002;
        /// disable movement
        const DISABLE_MOVE = 0x00000004;
        /// controlled by player, use IMMUNE_TO_PC instead of IMMUNE_TO_NPC
        const PLAYER_CONTROLLED = 0x00000008;
        const RENAME = 0x00000010;
        /// don't take reagents for spells with SPELL_ATTR5_NO_REAGENT_COST_WITH_AURA
        const PREPARATION = 0x00000020;
        const UNK_6 = 0x00000040;
        /// UNIT_FLAG_PLAYER_CONTROLLED | NOT_ATTACKABLE_1 is NON_PVP_ATTACKABLE
        const NOT_ATTACKABLE_1 = 0x00000080;
        /// disables combat/assistance with PlayerCharacters
        const IMMUNE_TO_PC = 0x00000100;
        /// disables combat/assistance with NonPlayerCharacters
        const IMMUNE_TO_NPC = 0x00000200;
        /// loot animation
        const LOOTING = 0x00000400;
        /// in combat?, 2.0.8
        const PET_IN_COMBAT = 0x00000800;
        /// changed in 3.0.3
        const PVP = 0x00001000;
        /// silenced, 2.1.1
        const SILENCED = 0x00002000;
        /// 2.0.8
        const CANNOT_SWIM = 0x00004000;
        /// shows swim animation in water
        const SWIMMING = 0x00008000;
        /// removes attackable icon, added by SPELL_AURA_MOD_UNATTACKABLE
        const NON_ATTACKABLE_2 = 0x00010000;
        /// 3.0.3 ok
        const PACIFIED = 0x00020000;
        /// 3.0.3 ok
        const STUNNED = 0x00040000;
        const IN_COMBAT = 0x00080000;
        /// disable casting at client side spell not allowed by taxi flight
        const TAXI_FLIGHT = 0x00100000;
        /// 3.0.3, disable melee spells casting
        const DISARMED = 0x00200000;
        const CONFUSED = 0x00400000;
        const FLEEING = 0x00800000;
        /// under direct client control by a player (possess or vehicle)
        const POSSESSED = 0x01000000;
        const NOT_SELECTABLE = 0x02000000;
        const SKINNABLE = 0x04000000;
        const MOUNT = 0x08000000;
        const UNK_28 = 0x10000000;
        /// Prevent automatically playing emotes from parsing chat text
        const PREVENT_EMOTES_FROM_CHAT_TEXT = 0x20000000;
        const SHEATHE = 0x40000000;
        /// Immune to damage
        const IMMUNE = 0x80000000;
    }
}
