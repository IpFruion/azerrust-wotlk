use std::num::NonZeroU8;

use crate::conditions::{
    global::GlobalConditionKind, multi_target::MultiTargetConditionKind,
    player::PlayerConditionKind, unit::UnitConditionKind, world_object::WorldObjectConditionKind,
};

macro_rules! match_try_cond {
    ($kind:expr, $values:expr, $(($v:expr => $ck:ident, $cki:path)),* $(,)?) => {{
        match $kind {
            $(
                $v => Ok(ConditionKind::$ck($cki(TryInto::try_into($values)?))),
            )*
            _ => Err(()),
        }
    }};
}

pub enum ConditionKind {
    Unit(UnitConditionKind),
    WorldObject(WorldObjectConditionKind),
    Player(PlayerConditionKind),
    Global(GlobalConditionKind),
    MultiTarget(MultiTargetConditionKind),
}

impl TryFrom<(NonZeroU8, (u32, u32, u32))> for ConditionKind {
    type Error = ();

    fn try_from((cond, values): (NonZeroU8, (u32, u32, u32))) -> Result<Self, Self::Error> {
        //TODO: Make this an array lookup instead of a switch for fast lookup
        match_try_cond!(
            cond.get(), values,
            (1   => Unit, UnitConditionKind::Aura),
            (15  => Unit, UnitConditionKind::Class),
            (16  => Unit, UnitConditionKind::Race),
            (21  => Unit, UnitConditionKind::UnitState),
            (27  => Unit, UnitConditionKind::Level),
            (36  => Unit, UnitConditionKind::Alive),
            (37  => Unit, UnitConditionKind::HpVal),
            (38  => Unit, UnitConditionKind::HpPct),
            (40  => Unit, UnitConditionKind::InWater),
            (42  => Unit, UnitConditionKind::StandState),
            (44  => Unit, UnitConditionKind::Charmed),
            (102 => Unit, UnitConditionKind::HasAuraType),
            (106 => Unit, UnitConditionKind::InCombat),
            (4   => WorldObject, WorldObjectConditionKind::ZoneId),
            (19  => WorldObject, WorldObjectConditionKind::SpawnMask),
            (22  => WorldObject, WorldObjectConditionKind::MapId),
            (23  => WorldObject, WorldObjectConditionKind::AreaId),
            (26  => WorldObject, WorldObjectConditionKind::PhaseMask),
            (32  => WorldObject, WorldObjectConditionKind::TypeMask),
            (2   => Player, PlayerConditionKind::Item),
            (3   => Player, PlayerConditionKind::ItemEquipped),
            (5   => Player, PlayerConditionKind::ReputationRank),
            (6   => Player, PlayerConditionKind::Team),
            (7   => Player, PlayerConditionKind::Skill),
            (8   => Player, PlayerConditionKind::QuestRewarded),
            (9   => Player, PlayerConditionKind::QuestTaken),
            (10  => Player, PlayerConditionKind::DrunkenState),
            (14  => Player, PlayerConditionKind::QuestNone),
            (17  => Player, PlayerConditionKind::Achievement),
            (18  => Player, PlayerConditionKind::Title),
            (20  => Player, PlayerConditionKind::Gender),
            (25  => Player, PlayerConditionKind::Spell),
            (28  => Player, PlayerConditionKind::QuestComplete),
            (43  => Player, PlayerConditionKind::DailyQuestDone),
            (45  => Player, PlayerConditionKind::PetType),
            (46  => Player, PlayerConditionKind::Taxi),
            (47  => Player, PlayerConditionKind::QuestState),
            (48  => Player, PlayerConditionKind::QuestObjectiveProgress),
            (101 => Player, PlayerConditionKind::QuestSatisfyExclusive),
            (105 => Player, PlayerConditionKind::PlayerQueuedRandomDungeon),
            (12  => Global, GlobalConditionKind::ActiveEvent),
            (11  => Global, GlobalConditionKind::WorldState),
            (103 => Global, GlobalConditionKind::WorldScript),
            (39  => Global, GlobalConditionKind::RealmAchievement),
            (24  => WorldObject, WorldObjectConditionKind::CreatureType),
            (49  => WorldObject, WorldObjectConditionKind::DifficultyId),
            (29  => WorldObject, WorldObjectConditionKind::NearCreature),
            (30  => WorldObject, WorldObjectConditionKind::NearGameObject),
            (31  => WorldObject, WorldObjectConditionKind::ObjectEntryGuid),
            (13  => WorldObject, WorldObjectConditionKind::InstanceInfo),
            (104 => WorldObject, WorldObjectConditionKind::AiData),
            (33  => MultiTarget, MultiTargetConditionKind::Relation),
            (34  => MultiTarget, MultiTargetConditionKind::Reaction),
            (35  => MultiTarget, MultiTargetConditionKind::Distance),
        )
    }
}
