/*
 * This file is part of the AzerothCore Project. See AUTHORS file for Copyright
 * information
 *
 * This program is free software; you can redistribute it and/or modify it
 * under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
 * FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for
 * more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program. If not, see <http://www.gnu.org/licenses/>.
 */

#include "AllMapScript.h"
#include "Config.h"
#include "DBCEnums.h"
#include "Log.h"
#include "Map.h"
#include "Player.h"
#include "PlayerScript.h"
#include "SpellAuras.h"
#include "SpellDefines.h"
#include "SpellAuraEffects.h"
#include "SpellMgr.h"
#include "Unit.h"

static uint32 constexpr FLEX_STAT_SPELL_ID = 100103;

struct FlexCategory {
  uint32 spellId;
  uint32 ratePercent;
};

class flex : public AllMapScript {
public:
  flex() : AllMapScript("flex") {
    int32 normalDungeonRaw =
        sConfigMgr->GetOption<int32>("Flex.NormalDungeonPercent", 5);
    int32 heroicDungeonRaw =
        sConfigMgr->GetOption<int32>("Flex.HeroicDungeonPercent", 7);
    int32 normalRaidRaw =
        sConfigMgr->GetOption<int32>("Flex.NormalRaidPercent", 10);
    int32 heroicRaidRaw =
        sConfigMgr->GetOption<int32>("Flex.HeroicRaidPercent", 13);
    int32 mythicRaidRaw =
        sConfigMgr->GetOption<int32>("Flex.MythicRaidPercent", 15);

    _normalDungeonPercent =
        normalDungeonRaw < 0 ? 0u : static_cast<uint32>(normalDungeonRaw);
    _heroicDungeonPercent =
        heroicDungeonRaw < 0 ? 0u : static_cast<uint32>(heroicDungeonRaw);
    _normalRaidPercent =
        normalRaidRaw < 0 ? 0u : static_cast<uint32>(normalRaidRaw);
    _heroicRaidPercent =
        heroicRaidRaw < 0 ? 0u : static_cast<uint32>(heroicRaidRaw);
    _mythicRaidPercent =
        mythicRaidRaw < 0 ? 0u : static_cast<uint32>(mythicRaidRaw);

    LOG_INFO("module.flex", "Flex loaded with rates: dungeon {}%, heroic dungeon {}%, raid {}%, heroic raid {}%, mythic raid {}%",
        _normalDungeonPercent, _heroicDungeonPercent, _normalRaidPercent, _heroicRaidPercent, _mythicRaidPercent);

    if (!sSpellMgr->GetSpellInfo(FLEX_STAT_SPELL_ID))
      LOG_ERROR("module.flex", "Flex Stat spell {} was not found; the pending world SQL update may not be imported", FLEX_STAT_SPELL_ID);
  }

  void OnPlayerEnterAll(Map *map, Player *player) override {
    if (!map || !player)
      return;

    InstanceMap *instance = map->ToInstanceMap();
    if (!instance)
      return;

    RecalculateBuffs(instance);
  }

  void OnPlayerLeaveAll(Map *map, Player *player) override {
    if (!map || !player)
      return;

    InstanceMap *instance = map->ToInstanceMap();
    if (!instance)
      return;

    // Remove the module aura from the leaving player immediately.
    if (player->HasAura(FLEX_STAT_SPELL_ID))
    {
        LOG_INFO("module.flex", "Flex Stat removed from leaving player (GUID {})", player->GetGUID().ToString());
        player->RemoveAura(FLEX_STAT_SPELL_ID);
    }

    RecalculateBuffs(instance);
  }

private:
  /// Select the rate for the given instance map.
  /// Returns {0, 0} for unsupported maps/difficulties.
  FlexCategory SelectCategoryAndRate(InstanceMap const *instance) const {
    Difficulty difficulty = instance->GetDifficulty();

    if (instance->IsNonRaidDungeon()) {
      if (difficulty == DUNGEON_DIFFICULTY_NORMAL)
        return {FLEX_STAT_SPELL_ID, _normalDungeonPercent};

      if (difficulty == DUNGEON_DIFFICULTY_HEROIC)
        return {FLEX_STAT_SPELL_ID, _heroicDungeonPercent};

      // Other dungeon difficulties (e.g. EPIC) are unsupported.
      return {0, 0};
    }

    if (instance->IsRaid()) {
      if (difficulty <= RAID_DIFFICULTY_25MAN_NORMAL)
        return {FLEX_STAT_SPELL_ID, _normalRaidPercent};

      if (difficulty <= RAID_DIFFICULTY_25MAN_HEROIC)
        return {FLEX_STAT_SPELL_ID, _heroicRaidPercent};

      // Custom raid difficulty values greater than RAID_DIFFICULTY_25MAN_HEROIC
      // use mythic.
      return {FLEX_STAT_SPELL_ID, _mythicRaidPercent};
    }

    // Unsupported map type.
    return {0, 0};
  }

  /// Count active non-GM players on the instance.
  static uint32 CountActiveNonGmPlayers(InstanceMap const *instance) {
    uint32 count = 0;
    for (Map::PlayerList::const_iterator itr = instance->GetPlayers().begin();
         itr != instance->GetPlayers().end(); ++itr) {
      Player *plr = itr->GetSource();
      if (plr && !plr->IsGameMaster())
        ++count;
    }
    return count;
  }

  /// Remove the module aura from every player on the instance (including GMs).
  static void RemoveAllModuleAuras(InstanceMap const *instance) {
      for (Map::PlayerList::const_iterator itr = instance->GetPlayers().begin();
           itr != instance->GetPlayers().end(); ++itr) {
        if (Player *plr = itr->GetSource())
          if (plr->HasAura(FLEX_STAT_SPELL_ID)) {
            LOG_INFO("module.flex", "Flex Stat removed from player (GUID {}) for recalculation", plr->GetGUID().ToString());
            plr->RemoveAura(FLEX_STAT_SPELL_ID);
            plr->UpdateAllStats();
          }
    }
  }

  /// Recalculate and apply Flex Stat for the given instance.
  void RecalculateBuffs(InstanceMap *instance) {
    FlexCategory cat = SelectCategoryAndRate(instance);
    if (cat.spellId == 0) {
      RemoveAllModuleAuras(instance);
      LOG_INFO("module.flex", "Flex ignored unsupported map {} difficulty {}", instance->GetId(), instance->GetDifficulty());
      return;
    }

    uint32 const maxPlayers = instance->GetMaxPlayers();
    uint32 const activePlayers = CountActiveNonGmPlayers(instance);
    uint32 const missingPlayers =
        maxPlayers > activePlayers ? maxPlayers - activePlayers : 0;
    uint64 const rawBonus =
        static_cast<uint64>(missingPlayers) * cat.ratePercent;
    int32 const bonusPercent = static_cast<int32>(rawBonus);

    LOG_INFO("module.flex", "Flex Stat recalculated for map {} instance {}: {} eligible players / {} maximum, {}% bonus",
        instance->GetId(), instance->GetInstanceId(), activePlayers, maxPlayers, bonusPercent);

    if (bonusPercent <= 0) {
      // No bonus to apply, but still ensure no stale auras remain.
      RemoveAllModuleAuras(instance);
      return;
    }

    // First strip all module auras from every player (including GMs).
    RemoveAllModuleAuras(instance);

    // Apply the selected aura to non-GM players with the computed bonus.
    for (Map::PlayerList::const_iterator itr = instance->GetPlayers().begin();
         itr != instance->GetPlayers().end(); ++itr) {
      Player *plr = itr->GetSource();
      if (!plr || plr->IsGameMaster())
        continue;

      SpellCastResult const result = plr->CastCustomSpell(
          cat.spellId, SPELLVALUE_BASE_POINT0, bonusPercent, plr, true);
      if (result != SPELL_CAST_OK)
        LOG_ERROR("module.flex",
                  "Failed to cast spell {} on player (GUID {}): result {}",
                  cat.spellId, plr->GetGUID().ToString(), result);
      else
      {
        plr->UpdateAllStats();

        if (AuraEffect const* effect = plr->GetAuraEffect(FLEX_STAT_SPELL_ID, EFFECT_0))
          LOG_INFO("module.flex", "Flex Stat aura confirmed on player (GUID {}): {}%; health {}/{}, total stamina {}",
              plr->GetGUID().ToString(), effect->GetAmount(), plr->GetHealth(), plr->GetMaxHealth(), plr->GetTotalStatValue(STAT_STAMINA));
        else
          LOG_ERROR("module.flex", "Flex Stat cast succeeded but aura {} was not found on player (GUID {})", FLEX_STAT_SPELL_ID, plr->GetGUID().ToString());
      }
    }
  }

  uint32 _normalDungeonPercent;
  uint32 _heroicDungeonPercent;
  uint32 _normalRaidPercent;
  uint32 _heroicRaidPercent;
  uint32 _mythicRaidPercent;
};

class flex_player : public PlayerScript {
public:
  flex_player() : PlayerScript("flex_player") { }

  void OnPlayerMapChanged(Player *player) override {
    if (!player || !player->GetMap() || player->GetMap()->ToInstanceMap())
      return;

    if (player->HasAura(FLEX_STAT_SPELL_ID)) {
      LOG_INFO("module.flex", "Flex Stat removed after leaving instance (GUID {})", player->GetGUID().ToString());
      player->RemoveAura(FLEX_STAT_SPELL_ID);
      player->UpdateAllStats();
    }
  }
};

void Addmod_flexScripts() {
  new flex();
  new flex_player();
}
