#pragma once
#include "AchievementMgr.h"
#include "ConditionMgr.h"
#include "Creature.h"
#include "DBCStores.h"
#include "GameEventMgr.h"
#include "GameObject.h"
#include "GameObjectAI.h"
#include "InstanceScript.h"
#include "LFGMgr.h"
#include "Map.h"
#include "Object.h"
#include "ObjectMgr.h"
#include "Pet.h"
#include "Player.h"
#include "ReputationMgr.h"
#include "ScriptedCreature.h"
#include "SpellMgr.h"
#include "Unit.h"
#include "WorldState.h"

// Needed because HasAuraEffect has 3 arguments normally but has a default
// parameter so using this instead for now
inline bool azerrust_unit_has_aura_effect(const Unit &unit, uint32_t spellId,
                                          uint8_t effIndex) {
  return unit.HasAuraEffect(spellId, effIndex);
}

// Needed because of the static cast from the u32 type, Enum not drafted yet
inline bool azerrust_unit_has_aura_type(const Unit &unit, uint32_t auraType) {
  return unit.HasAuraType(static_cast<AuraType>(auraType));
}

inline uint32_t azerrust_creaturetemplate_get_type(const CreatureTemplate &ct) {
  return ct.type;
}

inline uint8_t azerrust_player_get_quest_status(const Player &player,
                                                uint32_t questId) {
  return static_cast<uint8_t>(player.GetQuestStatus(questId));
}

inline uint8_t azerrust_player_get_team_id(const Player &player) {
  return static_cast<uint8_t>(player.GetTeamId());
}

inline uint8_t azerrust_pet_get_pet_type(const Pet &pet) {
  return static_cast<uint8_t>(pet.getPetType());
}

inline bool azerrust_player_queued_random_dungeon(const Player &player,
                                                  uint32_t checkDifficulty,
                                                  uint32_t difficulty) {
  if (!sLFGMgr->IsPlayerQueuedForRandomDungeon(player.GetGUID()))
    return false;
  if (!checkDifficulty)
    return true;
  return player.GetMap()->GetDifficulty() ==
         static_cast<Difficulty>(difficulty);
}

// Singleton accessors
inline WorldState *azerrust_world_state() { return sWorldState; }

inline GameEventMgr *azerrust_game_event_mgr() { return sGameEventMgr; }

inline ObjectMgr *azerrust_object_mgr() { return sObjectMgr; }

inline FactionEntry const *azerrust_faction_store_lookup(uint32_t id) {
  return sFactionStore.LookupEntry(id);
}

inline uint8_t azerrust_player_reputation_rank(const Player &player,
                                               const FactionEntry *faction) {
  return static_cast<uint8_t>(player.GetReputationMgr().GetRank(faction));
}

inline uint8_t azerrust_game_object_go_state(const GameObject &gameobject) {
  return static_cast<uint8_t>(gameobject.GetGoState());
}

inline AchievementGlobalMgr *azerrust_achievement_mgr() {
  return sAchievementMgr;
}

inline AchievementEntry const *azerrust_achievement_store_lookup(uint32_t id) {
  return sAchievementStore.LookupEntry(id);
}

inline uint32_t azerrust_unit_has_owner(const WorldObject &obj,
                                                     uint32_t val1,
                                                     uint32_t val2,
                                                     uint32_t val3) {
  Map *map = const_cast<WorldObject &>(obj).GetMap();
  if (!map->IsDungeon())
    return false;
  InstanceScript const *instance = map->ToInstanceMap()->GetInstanceScript();
  if (!instance)
    return false;
  switch (val3) {
  case 0:
    return instance->GetData(val1) == val2;
  case 1:
    return instance->GetGuidData(val1) == ObjectGuid(uint64(val2));
  case 2:
    return instance->GetBossState(val1) == EncounterState(val2);
  case 3:
    return instance->GetData64(val1) == val2;
  }
  return false;
}

inline int8_t azerrust_worldobject_check_reaction(const WorldObject &obj,
                                                  const WorldObject &target) {
  Unit const *unitA = obj.ToUnit();
  Unit const *unitB = target.ToUnit();
  if (!unitA || !unitB)
    return -1;
  return static_cast<int8_t>(unitA->GetReactionTo(const_cast<Unit *>(unitB)));
}

inline float azerrust_worldobject_get_distance(const WorldObject &obj,
                                               const WorldObject &target) {
  return obj.GetDistance(&target);
}

inline bool azerrust_unit_has_owner(const Unit& unit, const Unit& owner) {
    return unit.GetOwnerGUID() == const_cast<Unit&>(owner).GetGUID();
}

inline bool azerrust_unit_has_creator(const Unit& unit, const Unit& creator) {
    return unit.GetCreatorGUID() == const_cast<Unit&>(creator).GetGUID();
}

inline const InstanceScript* azerrust_map_get_instance_script(const Map& map) {
    if (!map.IsDungeon()) return nullptr;
    return map.ToInstanceMap()->GetInstanceScript();
}

inline uint32_t azerrust_instance_script_get_boss_state(const InstanceScript& script, uint32_t bossId) {
    return static_cast<uint32_t>(script.GetBossState(bossId));
}

inline bool azerrust_instance_script_check_guid_data(const InstanceScript& script, uint32_t type, uint32_t expectedLow) {
    ObjectGuid guid = script.GetGuidData(type);
    return guid == ObjectGuid(uint64(expectedLow));
}

inline uint32_t azerrust_worldobject_get_type_id(const WorldObject &obj) {
  return static_cast<uint32_t>(obj.GetTypeId());
}

inline bool azerrust_unit_has_unit_flag(const Unit &unit, uint32_t flag) {
  return unit.HasUnitFlag(static_cast<UnitFlags>(flag));
}

inline WorldObject *azerrust_source_info_get_target(ConditionSourceInfo &info,
                                                    uint8_t index) {
  if (index >= MAX_CONDITION_TARGETS)
    return nullptr;
  return info.mConditionTargets[index];
}
