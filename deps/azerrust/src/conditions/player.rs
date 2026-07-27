use crate::{DrunkenState, Team, TeamId, entities::player::PlayerRef, quests::QuestStatus};
use enum_dispatch::enum_dispatch;

#[enum_dispatch(PlayerConditionTrait)]
pub enum PlayerConditionKind {
    Item(PlayerConditionItem),
    ItemEquipped(PlayerConditionItemEquipped),
    ReputationRank(PlayerConditionReputationRank),
    Team(PlayerConditionTeam),
    Skill(PlayerConditionSkill),
    QuestRewarded(PlayerConditionQuestRewarded),
    QuestTaken(PlayerConditionQuestTaken),
    QuestComplete(PlayerConditionQuestComplete),
    QuestNone(PlayerConditionQuestNone),
    QuestSatisfyExclusive(PlayerConditionQuestSatisfyExclusive),
    QuestState(PlayerConditionQuestState),
    QuestObjectiveProgress(PlayerConditionQuestObjectiveProgress),
    Achievement(PlayerConditionAchievement),
    Title(PlayerConditionTitle),
    Gender(PlayerConditionGender),
    Spell(PlayerConditionSpell),
    DrunkenState(PlayerConditionDrunkenState),
    DailyQuestDone(PlayerConditionDailyQuestDone),
    PetType(PlayerConditionPetType),
    Taxi(PlayerConditionTaxi),
    PlayerQueuedRandomDungeon(PlayerConditionPlayerQueuedRandomDungeon),
}

#[enum_dispatch]
pub trait PlayerConditionTrait {
    fn meets(&self, player: PlayerRef) -> Result<bool, ()>;
}

pub struct PlayerConditionItem {
    item_id: u32,
    count: u32,
    check_bank: bool,
}

impl PlayerConditionTrait for PlayerConditionItem {
    fn meets(&self, player: PlayerRef) -> Result<bool, ()> {
        Ok(player.HasItemCount(self.item_id, self.count, self.check_bank))
    }
}

impl TryFrom<(u32, u32, u32)> for PlayerConditionItem {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(PlayerConditionItem {
            item_id: value.0,
            count: value.1,
            check_bank: value.2 != 0,
        })
    }
}

pub struct PlayerConditionItemEquipped {
    item_id: u32,
}

impl PlayerConditionTrait for PlayerConditionItemEquipped {
    fn meets(&self, player: PlayerRef) -> Result<bool, ()> {
        Ok(player.HasItemOrGemWithIdEquipped(self.item_id, 1, 1))
    }
}

impl TryFrom<(u32, u32, u32)> for PlayerConditionItemEquipped {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(PlayerConditionItemEquipped { item_id: value.0 })
    }
}

pub struct PlayerConditionReputationRank {
    faction_id: u32,
    rank_mask: u32,
}

impl PlayerConditionTrait for PlayerConditionReputationRank {
    fn meets(&self, player: PlayerRef) -> Result<bool, ()> {
        let faction =
            unsafe { crate::ffi::azerrust_faction_store_lookup(self.faction_id).as_ref() };
        Ok(faction.is_some_and(|f| {
            let rank =
                unsafe { crate::ffi::azerrust_player_reputation_rank(&player, f as *const _) };
            self.rank_mask & (1 << rank) != 0
        }))
    }
}

impl TryFrom<(u32, u32, u32)> for PlayerConditionReputationRank {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(PlayerConditionReputationRank {
            faction_id: value.0,
            rank_mask: value.1,
        })
    }
}

pub struct PlayerConditionTeam {
    team: Team,
}

impl PlayerConditionTrait for PlayerConditionTeam {
    fn meets(&self, player: PlayerRef) -> Result<bool, ()> {
        let team_old = match player.team_id()? {
            TeamId::Alliance => Team::Alliance,
            _ => Team::Horde,
        };
        Ok(team_old == self.team)
    }
}

impl TryFrom<(u32, u32, u32)> for PlayerConditionTeam {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(PlayerConditionTeam {
            team: Team::from_repr(value.0).ok_or(())?,
        })
    }
}

pub struct PlayerConditionSkill {
    skill_id: u32,
    skill_value: u32,
}

impl PlayerConditionTrait for PlayerConditionSkill {
    fn meets(&self, player: PlayerRef) -> Result<bool, ()> {
        Ok(player.HasSkill(self.skill_id)
            && u32::from(player.GetBaseSkillValue(self.skill_id)) >= self.skill_value)
    }
}

impl TryFrom<(u32, u32, u32)> for PlayerConditionSkill {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(PlayerConditionSkill {
            skill_id: value.0,
            skill_value: value.1,
        })
    }
}

pub struct PlayerConditionQuestRewarded {
    quest_id: u32,
}

impl PlayerConditionTrait for PlayerConditionQuestRewarded {
    fn meets(&self, player: PlayerRef) -> Result<bool, ()> {
        Ok(player.GetQuestRewardStatus(self.quest_id))
    }
}

impl TryFrom<(u32, u32, u32)> for PlayerConditionQuestRewarded {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(PlayerConditionQuestRewarded { quest_id: value.0 })
    }
}

pub struct PlayerConditionQuestTaken {
    quest_id: u32,
}

impl PlayerConditionTrait for PlayerConditionQuestTaken {
    fn meets(&self, player: PlayerRef) -> Result<bool, ()> {
        Ok(player.quest_status(self.quest_id)? == QuestStatus::Incomplete)
    }
}

impl TryFrom<(u32, u32, u32)> for PlayerConditionQuestTaken {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(PlayerConditionQuestTaken { quest_id: value.0 })
    }
}

pub struct PlayerConditionQuestComplete {
    quest_id: u32,
}

impl PlayerConditionTrait for PlayerConditionQuestComplete {
    fn meets(&self, player: PlayerRef) -> Result<bool, ()> {
        Ok(player.quest_status(self.quest_id)? == QuestStatus::Complete
            && !player.GetQuestRewardStatus(self.quest_id))
    }
}

impl TryFrom<(u32, u32, u32)> for PlayerConditionQuestComplete {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(PlayerConditionQuestComplete { quest_id: value.0 })
    }
}

pub struct PlayerConditionQuestNone {
    quest_id: u32,
}

impl PlayerConditionTrait for PlayerConditionQuestNone {
    fn meets(&self, player: PlayerRef) -> Result<bool, ()> {
        Ok(player.quest_status(self.quest_id)? == QuestStatus::None)
    }
}

impl TryFrom<(u32, u32, u32)> for PlayerConditionQuestNone {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(PlayerConditionQuestNone { quest_id: value.0 })
    }
}

pub struct PlayerConditionQuestSatisfyExclusive {
    quest_id: u32,
}

impl PlayerConditionTrait for PlayerConditionQuestSatisfyExclusive {
    fn meets(&self, player: PlayerRef) -> Result<bool, ()> {
        if player.IsQuestRewarded(self.quest_id) {
            return Ok(false);
        }
        let quest = unsafe { crate::ffi::azerrust_object_mgr().as_ref() }
            .and_then(|mgr| unsafe { mgr.GetQuestTemplate(self.quest_id).as_ref() });
        Ok(quest
            .is_some_and(|q| unsafe { player.SatisfyQuestExclusiveGroup(q as *const _, false) }))
    }
}

impl TryFrom<(u32, u32, u32)> for PlayerConditionQuestSatisfyExclusive {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(PlayerConditionQuestSatisfyExclusive { quest_id: value.0 })
    }
}

pub struct PlayerConditionQuestState {
    quest_id: u32,
    state_mask: u32,
}

impl PlayerConditionTrait for PlayerConditionQuestState {
    fn meets(&self, player: PlayerRef) -> Result<bool, ()> {
        let status = player.quest_status(self.quest_id)?;
        let mask = self.state_mask;
        if (mask & (1 << 0)) != 0 && status == QuestStatus::None {
            return Ok(true);
        }
        if (mask & (1 << 1)) != 0 && status == QuestStatus::Complete {
            return Ok(true);
        }
        if (mask & (1 << 2)) != 0 && status == QuestStatus::Incomplete {
            return Ok(true);
        }
        if (mask & (1 << 3)) != 0 && status == QuestStatus::Failed {
            return Ok(true);
        }
        if (mask & (1 << 6)) != 0 && player.GetQuestRewardStatus(self.quest_id) {
            return Ok(true);
        }
        Ok(false)
    }
}

impl TryFrom<(u32, u32, u32)> for PlayerConditionQuestState {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(PlayerConditionQuestState {
            quest_id: value.0,
            state_mask: value.1,
        })
    }
}

pub struct PlayerConditionQuestObjectiveProgress {
    quest_id: u32,
    objective_index: u32,
    objective_count: u32,
}

impl PlayerConditionTrait for PlayerConditionQuestObjectiveProgress {
    fn meets(&self, player: PlayerRef) -> Result<bool, ()> {
        let quest = unsafe { crate::ffi::azerrust_object_mgr().as_ref() }
            .and_then(|mgr| unsafe { mgr.GetQuestTemplate(self.quest_id).as_ref() });
        let Some(quest) = quest else { return Ok(false) };
        let quest_id = quest.GetQuestId();
        let log_slot = player.FindQuestSlot(quest_id);
        if log_slot >= 175 {
            return Ok(false);
        }
        Ok(
            u32::from(player.GetQuestSlotCounter(log_slot, self.objective_index as u8))
                == self.objective_count,
        )
    }
}

impl TryFrom<(u32, u32, u32)> for PlayerConditionQuestObjectiveProgress {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(PlayerConditionQuestObjectiveProgress {
            quest_id: value.0,
            objective_index: value.1,
            objective_count: value.2,
        })
    }
}

pub struct PlayerConditionAchievement {
    achievement_id: u32,
}

impl PlayerConditionTrait for PlayerConditionAchievement {
    fn meets(&self, player: PlayerRef) -> Result<bool, ()> {
        Ok(player.HasAchieved(self.achievement_id))
    }
}

impl TryFrom<(u32, u32, u32)> for PlayerConditionAchievement {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(PlayerConditionAchievement {
            achievement_id: value.0,
        })
    }
}

pub struct PlayerConditionTitle {
    title_id: u32,
}

impl PlayerConditionTrait for PlayerConditionTitle {
    fn meets(&self, player: PlayerRef) -> Result<bool, ()> {
        Ok(player.HasTitle(self.title_id))
    }
}

impl TryFrom<(u32, u32, u32)> for PlayerConditionTitle {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(PlayerConditionTitle { title_id: value.0 })
    }
}

pub struct PlayerConditionGender {
    gender: u32,
}

impl PlayerConditionTrait for PlayerConditionGender {
    fn meets(&self, player: PlayerRef) -> Result<bool, ()> {
        Ok(u32::from(player.getGender()) == self.gender)
    }
}

impl TryFrom<(u32, u32, u32)> for PlayerConditionGender {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(PlayerConditionGender { gender: value.0 })
    }
}

pub struct PlayerConditionSpell {
    spell_id: u32,
}

impl PlayerConditionTrait for PlayerConditionSpell {
    fn meets(&self, player: PlayerRef) -> Result<bool, ()> {
        Ok(player.HasSpell(self.spell_id))
    }
}

impl TryFrom<(u32, u32, u32)> for PlayerConditionSpell {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(PlayerConditionSpell { spell_id: value.0 })
    }
}

pub struct PlayerConditionDrunkenState {
    min_state: DrunkenState,
}

impl PlayerConditionTrait for PlayerConditionDrunkenState {
    fn meets(&self, player: PlayerRef) -> Result<bool, ()> {
        Ok(DrunkenState::from_value(player.GetDrunkValue()) >= self.min_state)
    }
}

impl TryFrom<(u32, u32, u32)> for PlayerConditionDrunkenState {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(PlayerConditionDrunkenState {
            min_state: DrunkenState::from_repr(value.0).ok_or(())?,
        })
    }
}

pub struct PlayerConditionDailyQuestDone {
    quest_id: u32,
}

impl PlayerConditionTrait for PlayerConditionDailyQuestDone {
    fn meets(&self, mut player: PlayerRef) -> Result<bool, ()> {
        Ok(player.as_mut().IsDailyQuestDone(self.quest_id))
    }
}

impl TryFrom<(u32, u32, u32)> for PlayerConditionDailyQuestDone {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(PlayerConditionDailyQuestDone { quest_id: value.0 })
    }
}

pub struct PlayerConditionPetType {
    mask: u32,
}

impl PlayerConditionTrait for PlayerConditionPetType {
    fn meets(&self, mut player: PlayerRef) -> Result<bool, ()> {
        let pet = player.pet();
        Ok(pet.is_some_and(|p| (1 << crate::ffi::azerrust_pet_get_pet_type(&p)) & self.mask != 0))
    }
}

impl TryFrom<(u32, u32, u32)> for PlayerConditionPetType {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(PlayerConditionPetType { mask: value.0 })
    }
}

pub struct PlayerConditionTaxi;

impl PlayerConditionTrait for PlayerConditionTaxi {
    fn meets(&self, player: PlayerRef) -> Result<bool, ()> {
        Ok(player.IsInFlight())
    }
}

impl TryFrom<(u32, u32, u32)> for PlayerConditionTaxi {
    type Error = ();

    fn try_from(_value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(PlayerConditionTaxi)
    }
}

pub struct PlayerConditionPlayerQueuedRandomDungeon {
    check_difficulty: u32,
    difficulty: u32,
}

impl PlayerConditionTrait for PlayerConditionPlayerQueuedRandomDungeon {
    fn meets(&self, player: PlayerRef) -> Result<bool, ()> {
        Ok(crate::ffi::azerrust_player_queued_random_dungeon(
            &player,
            self.check_difficulty,
            self.difficulty,
        ))
    }
}

impl TryFrom<(u32, u32, u32)> for PlayerConditionPlayerQueuedRandomDungeon {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(PlayerConditionPlayerQueuedRandomDungeon {
            check_difficulty: value.0,
            difficulty: value.1,
        })
    }
}
