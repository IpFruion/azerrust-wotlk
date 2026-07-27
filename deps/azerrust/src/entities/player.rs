use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
};

use crate::{quests::QuestStatus, TeamId};

#[cxx::bridge]
pub mod ffi {
    extern "C++" {
        type Pet = crate::entities::pet::ffi::Pet;
        type Quest = crate::quests::ffi::Quest;
        type ReputationMgr = crate::reputation::ffi::ReputationMgr;
    }

    unsafe extern "C++" {
        include!("Player.h");
        type Player;

        fn HasItemCount(&self, item: u32, count: u32, checkBank: bool) -> bool;
        fn HasItemOrGemWithIdEquipped(&self, item: u32, count: u32, checkGem: u8) -> bool;
        fn HasSkill(&self, skill: u32) -> bool;
        fn GetBaseSkillValue(&self, skill: u32) -> u16;
        fn GetQuestRewardStatus(&self, quest: u32) -> bool;
        fn HasAchieved(&self, achievement: u32) -> bool;
        fn HasTitle(&self, title: u32) -> bool;
        fn HasSpell(&self, spell: u32) -> bool;
        fn IsInFlight(&self) -> bool;
        fn getGender(&self) -> u8;
        fn IsDailyQuestDone(self: Pin<&mut Self>, quest_id: u32) -> bool;
        fn GetDrunkValue(&self) -> u8;
        fn FindQuestSlot(&self, quest_id: u32) -> u16;
        fn GetQuestSlotCounter(&self, slot: u16, objective_index: u8) -> u16;
        unsafe fn SatisfyQuestExclusiveGroup(&self, quest: *const Quest, is_accept: bool) -> bool;
        fn IsQuestRewarded(&self, quest_id: u32) -> bool;
        unsafe fn GetPet(&self) -> *mut Pet;
        // unsafe fn GetReputationMgr(&self) -> *const ReputationMgr;
    }
}

pub struct PlayerRef<'a>(pub Pin<&'a mut ffi::Player>);

impl<'a> Deref for PlayerRef<'a> {
    type Target = Pin<&'a mut ffi::Player>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for PlayerRef<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> PlayerRef<'a> {
    pub fn quest_status(&self, quest_id: u32) -> Result<QuestStatus, ()> {
        let raw = crate::ffi::azerrust_player_get_quest_status(&self.0, quest_id);
        QuestStatus::from_repr(raw).ok_or(())
    }

    pub fn team_id(&self) -> Result<TeamId, ()> {
        TeamId::from_repr(crate::ffi::azerrust_player_get_team_id(&self.0)).ok_or(())
    }

    pub fn pet(&mut self) -> Option<Pin<&mut ffi::Pet>> {
        let pet = unsafe { self.0.GetPet().as_mut() }?;
        Some(unsafe { Pin::new_unchecked(pet) })
    }
}
