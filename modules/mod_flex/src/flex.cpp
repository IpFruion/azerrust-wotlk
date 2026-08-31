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
#include "Pet.h"
#include "Player.h"
#include "PlayerScript.h"
#include "SpellAuras.h"
#include "SpellDefines.h"
#include "SpellAuraEffects.h"
#include "SpellMgr.h"
#include "Unit.h"
#include "Vehicle.h"

static uint32 constexpr FLEX_STAT_SPELL_ID = 100103;
static uint32 constexpr FLEX_SPELLPOWER_SPELL_ID = 100104;

/// Remove FLEX_SPELLPOWER_SPELL_ID from pet and vehicle base of the given player.
static void RemoveFlexSpellPowerFromControlled(Player* player)
{
    if (!player)
        return;

    if (Pet* pet = player->GetPet())
        pet->RemoveAura(FLEX_SPELLPOWER_SPELL_ID);

    if (Vehicle* vehicle = player->GetVehicle())
        if (Unit* base = vehicle->GetBase())
            base->RemoveAura(FLEX_SPELLPOWER_SPELL_ID);
}

struct FlexCategory {
  uint32 spellId;
  uint32 ratePercent;
  uint32 spellPowerPercent;
};

class flex : public AllMapScript {
public:
  flex() : AllMapScript("flex") {
    int32 normalDungeonRaw =
        sConfigMgr->GetOption<int32>("Flex.Normal.Dungeon.Stat.Percent", 5);
    int32 heroicDungeonRaw =
        sConfigMgr->GetOption<int32>("Flex.Heroic.Dungeon.Stat.Percent", 7);
    int32 normalRaidRaw =
        sConfigMgr->GetOption<int32>("Flex.Normal.Raid.Stat.Percent", 10);
    int32 heroicRaidRaw =
        sConfigMgr->GetOption<int32>("Flex.Heroic.Raid.Stat.Percent", 13);
    int32 mythicRaidRaw =
        sConfigMgr->GetOption<int32>("Flex.Mythic.Raid.Stat.Percent", 15);

    int32 normalDungeonSpellPowerRaw =
        sConfigMgr->GetOption<int32>("Flex.Normal.Dungeon.SpellPower.Percent", 5);
    int32 heroicDungeonSpellPowerRaw =
        sConfigMgr->GetOption<int32>("Flex.Heroic.Dungeon.SpellPower.Percent", 7);
    int32 normalRaidSpellPowerRaw =
        sConfigMgr->GetOption<int32>("Flex.Normal.Raid.SpellPower.Percent", 10);
    int32 heroicRaidSpellPowerRaw =
        sConfigMgr->GetOption<int32>("Flex.Heroic.Raid.SpellPower.Percent", 13);
    int32 mythicRaidSpellPowerRaw =
        sConfigMgr->GetOption<int32>("Flex.Mythic.Raid.SpellPower.Percent", 15);

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

    _normalDungeonSpellPowerPercent =
        normalDungeonSpellPowerRaw < 0 ? 0u : static_cast<uint32>(normalDungeonSpellPowerRaw);
    _heroicDungeonSpellPowerPercent =
        heroicDungeonSpellPowerRaw < 0 ? 0u : static_cast<uint32>(heroicDungeonSpellPowerRaw);
    _normalRaidSpellPowerPercent =
        normalRaidSpellPowerRaw < 0 ? 0u : static_cast<uint32>(normalRaidSpellPowerRaw);
    _heroicRaidSpellPowerPercent =
        heroicRaidSpellPowerRaw < 0 ? 0u : static_cast<uint32>(heroicRaidSpellPowerRaw);
    _mythicRaidSpellPowerPercent =
        mythicRaidSpellPowerRaw < 0 ? 0u : static_cast<uint32>(mythicRaidSpellPowerRaw);

    LOG_INFO("module.flex", "Flex loaded with stat rates: dungeon {}%, heroic dungeon {}%, raid {}%, heroic raid {}%, mythic raid {}%; spell power rates: dungeon {}%, heroic dungeon {}%, raid {}%, heroic raid {}%, mythic raid {}%",
        _normalDungeonPercent, _heroicDungeonPercent, _normalRaidPercent, _heroicRaidPercent, _mythicRaidPercent,
        _normalDungeonSpellPowerPercent, _heroicDungeonSpellPowerPercent, _normalRaidSpellPowerPercent, _heroicRaidSpellPowerPercent, _mythicRaidSpellPowerPercent);

    if (!sSpellMgr->GetSpellInfo(FLEX_STAT_SPELL_ID))
      LOG_ERROR("module.flex", "Flex Stat spell {} was not found; the pending world SQL update may not be imported", FLEX_STAT_SPELL_ID);

    if (!sSpellMgr->GetSpellInfo(FLEX_SPELLPOWER_SPELL_ID))
      LOG_ERROR("module.flex", "Flex SpellPower spell {} was not found; the pending world SQL update may not be imported", FLEX_SPELLPOWER_SPELL_ID);
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

    // Remove the module auras from the leaving player immediately.
    bool hadAura = false;
    if (player->HasAura(FLEX_STAT_SPELL_ID))
    {
        LOG_INFO("module.flex", "Flex Stat removed from leaving player (GUID {})", player->GetGUID().ToString());
        player->RemoveAura(FLEX_STAT_SPELL_ID);
        hadAura = true;
    }
    if (player->HasAura(FLEX_SPELLPOWER_SPELL_ID))
    {
        LOG_INFO("module.flex", "Flex SpellPower removed from leaving player (GUID {})", player->GetGUID().ToString());
        player->RemoveAura(FLEX_SPELLPOWER_SPELL_ID);
        hadAura = true;
    }
    if (hadAura)
        player->UpdateAllStats();

    RemoveFlexSpellPowerFromControlled(player);

    RecalculateBuffs(instance);
  }

private:
  /// Select the rate for the given instance map.
  /// Returns {0, 0, 0} for unsupported maps/difficulties.
  FlexCategory SelectCategoryAndRate(InstanceMap const *instance) const {
    Difficulty difficulty = instance->GetDifficulty();

    if (instance->IsNonRaidDungeon()) {
      if (difficulty == DUNGEON_DIFFICULTY_NORMAL)
        return {FLEX_STAT_SPELL_ID, _normalDungeonPercent, _normalDungeonSpellPowerPercent};

      if (difficulty == DUNGEON_DIFFICULTY_HEROIC)
        return {FLEX_STAT_SPELL_ID, _heroicDungeonPercent, _heroicDungeonSpellPowerPercent};

      // Other dungeon difficulties (e.g. EPIC) are unsupported.
      return {0, 0, 0};
    }

    if (instance->IsRaid()) {
      if (difficulty <= RAID_DIFFICULTY_25MAN_NORMAL)
        return {FLEX_STAT_SPELL_ID, _normalRaidPercent, _normalRaidSpellPowerPercent};

      if (difficulty <= RAID_DIFFICULTY_25MAN_HEROIC)
        return {FLEX_STAT_SPELL_ID, _heroicRaidPercent, _heroicRaidSpellPowerPercent};

      // Custom raid difficulty values greater than RAID_DIFFICULTY_25MAN_HEROIC
      // use mythic.
      return {FLEX_STAT_SPELL_ID, _mythicRaidPercent, _mythicRaidSpellPowerPercent};
    }

    // Unsupported map type.
    return {0, 0, 0};
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

  /// Remove the module auras from every player on the instance (including GMs).
  static void RemoveAllModuleAuras(InstanceMap const *instance)
  {
    for (Map::PlayerList::const_iterator itr = instance->GetPlayers().begin();
         itr != instance->GetPlayers().end(); ++itr)
    {
      if (Player *plr = itr->GetSource())
      {
        if (plr->HasAura(FLEX_STAT_SPELL_ID))
        {
          LOG_INFO("module.flex", "Flex Stat removed from player (GUID {}) for recalculation", plr->GetGUID().ToString());
          plr->RemoveAura(FLEX_STAT_SPELL_ID);
        }
        if (plr->HasAura(FLEX_SPELLPOWER_SPELL_ID))
        {
          LOG_INFO("module.flex", "Flex SpellPower removed from player (GUID {}) for recalculation", plr->GetGUID().ToString());
          plr->RemoveAura(FLEX_SPELLPOWER_SPELL_ID);
        }
      }
    }
  }

  /// Recalculate and apply Flex auras for the given instance.
  void RecalculateBuffs(InstanceMap *instance) {
    FlexCategory cat = SelectCategoryAndRate(instance);
    if (cat.spellId == 0) {
      RemoveAllModuleAuras(instance);
      for (Map::PlayerList::const_iterator itr = instance->GetPlayers().begin();
           itr != instance->GetPlayers().end(); ++itr)
      {
        if (Player *plr = itr->GetSource())
          plr->UpdateAllStats();
      }
      LOG_INFO("module.flex", "Flex ignored unsupported map {} difficulty {}", instance->GetId(), instance->GetDifficulty());
      return;
    }

    uint32 const maxPlayers = instance->GetMaxPlayers();
    uint32 const activePlayers = CountActiveNonGmPlayers(instance);
    uint32 const missingPlayers =
        maxPlayers > activePlayers ? maxPlayers - activePlayers : 0;
    uint64 const rawStatBonus =
        static_cast<uint64>(missingPlayers) * cat.ratePercent;
    uint64 const rawSpellPowerBonus =
        static_cast<uint64>(missingPlayers) * cat.spellPowerPercent;
    int32 const statBonusPercent = static_cast<int32>(rawStatBonus);
    int32 const spellPowerBonusPercent = static_cast<int32>(rawSpellPowerBonus);

    LOG_INFO("module.flex", "Flex recalculated for map {} instance {}: {} eligible players / {} maximum, stat {}% bonus, spell power {}% bonus",
        instance->GetId(), instance->GetInstanceId(), activePlayers, maxPlayers, statBonusPercent, spellPowerBonusPercent);

    if (statBonusPercent <= 0 && spellPowerBonusPercent <= 0) {
      // No bonus to apply, but still ensure no stale auras remain.
      RemoveAllModuleAuras(instance);
      for (Map::PlayerList::const_iterator itr = instance->GetPlayers().begin();
           itr != instance->GetPlayers().end(); ++itr)
      {
        if (Player *plr = itr->GetSource())
          plr->UpdateAllStats();
      }
      return;
    }

    // First strip all module auras from every player (including GMs).
    RemoveAllModuleAuras(instance);

    // Explicitly remove the spell power aura from player pets and vehicle bases
    // before applying the new amount.
    for (Map::PlayerList::const_iterator itr = instance->GetPlayers().begin();
         itr != instance->GetPlayers().end(); ++itr)
    {
      if (Player *plr = itr->GetSource())
      {
        if (Pet *pet = plr->GetPet())
          pet->RemoveAura(FLEX_SPELLPOWER_SPELL_ID);

        if (Vehicle *vehicle = plr->GetVehicle())
          if (Unit *vehicleBase = vehicle->GetBase())
            vehicleBase->RemoveAura(FLEX_SPELLPOWER_SPELL_ID);
      }
    }

    // Apply the spell power aura first to non-GM players and their controlled units.
    if (spellPowerBonusPercent > 0)
    {
      for (Map::PlayerList::const_iterator itr = instance->GetPlayers().begin();
           itr != instance->GetPlayers().end(); ++itr)
      {
        Player *plr = itr->GetSource();
        if (!plr || plr->IsGameMaster())
          continue;

        // Apply to the player.
        SpellCastResult const result = plr->CastCustomSpell(
            FLEX_SPELLPOWER_SPELL_ID, SPELLVALUE_BASE_POINT0, spellPowerBonusPercent, plr, true);
        if (result != SPELL_CAST_OK)
          LOG_ERROR("module.flex",
                    "Failed to cast spell {} on player (GUID {}): result {}",
                    FLEX_SPELLPOWER_SPELL_ID, plr->GetGUID().ToString(), result);

        // Apply to pet if present. Pet casts the self-targeting spell on itself.
        if (Pet *pet = plr->GetPet())
        {
          SpellCastResult const petResult = pet->CastCustomSpell(
              FLEX_SPELLPOWER_SPELL_ID, SPELLVALUE_BASE_POINT0, spellPowerBonusPercent, pet, true);
          if (petResult != SPELL_CAST_OK)
            LOG_ERROR("module.flex",
                      "Failed to cast spell {} on pet (GUID {}): result {}",
                      FLEX_SPELLPOWER_SPELL_ID, pet->GetGUID().ToString(), petResult);
        }

        // Apply to vehicle base unit if present. Vehicle base casts the self-targeting spell on itself.
        if (Vehicle *vehicle = plr->GetVehicle())
          if (Unit *vehicleBase = vehicle->GetBase())
          {
            SpellCastResult const vehResult = vehicleBase->CastCustomSpell(
                FLEX_SPELLPOWER_SPELL_ID, SPELLVALUE_BASE_POINT0, spellPowerBonusPercent, vehicleBase, true);
            if (vehResult != SPELL_CAST_OK)
              LOG_ERROR("module.flex",
                        "Failed to cast spell {} on vehicle base (GUID {}): result {}",
                        FLEX_SPELLPOWER_SPELL_ID, vehicleBase->GetGUID().ToString(), vehResult);
          }
      }
    }

    // Apply the stat aura last to non-GM players.
    if (statBonusPercent > 0)
    {
      for (Map::PlayerList::const_iterator itr = instance->GetPlayers().begin();
           itr != instance->GetPlayers().end(); ++itr)
      {
        Player *plr = itr->GetSource();
        if (!plr || plr->IsGameMaster())
          continue;

        SpellCastResult const result = plr->CastCustomSpell(
            cat.spellId, SPELLVALUE_BASE_POINT0, statBonusPercent, plr, true);
        if (result != SPELL_CAST_OK)
          LOG_ERROR("module.flex",
                    "Failed to cast spell {} on player (GUID {}): result {}",
                    cat.spellId, plr->GetGUID().ToString(), result);
        else
        {
          plr->UpdateAllStats();

          if (AuraEffect const* effect = plr->GetAuraEffect(cat.spellId, EFFECT_0))
            LOG_INFO("module.flex", "Flex Stat aura confirmed on player (GUID {}): {}%; health {}/{}, total stamina {}",
                plr->GetGUID().ToString(), effect->GetAmount(), plr->GetHealth(), plr->GetMaxHealth(), plr->GetTotalStatValue(STAT_STAMINA));
          else
            LOG_ERROR("module.flex", "Flex Stat cast succeeded but aura {} was not found on player (GUID {})", cat.spellId, plr->GetGUID().ToString());
        }
       }
     }
    else
    {
      // Stat bonus is zero (spell power is positive, otherwise we returned early).
      // All non-GM players had their stat auras stripped above; refresh stats
      // to clear stale values.
      for (Map::PlayerList::const_iterator itr = instance->GetPlayers().begin();
           itr != instance->GetPlayers().end(); ++itr)
      {
        if (Player *plr = itr->GetSource())
          if (!plr->IsGameMaster())
            plr->UpdateAllStats();
      }
    }
  }

  uint32 _normalDungeonPercent;
  uint32 _heroicDungeonPercent;
  uint32 _normalRaidPercent;
  uint32 _heroicRaidPercent;
  uint32 _mythicRaidPercent;
  uint32 _normalDungeonSpellPowerPercent;
  uint32 _heroicDungeonSpellPowerPercent;
  uint32 _normalRaidSpellPowerPercent;
  uint32 _heroicRaidSpellPowerPercent;
  uint32 _mythicRaidSpellPowerPercent;
};

class flex_player : public PlayerScript {
public:
  flex_player() : PlayerScript("flex_player") { }

  void OnPlayerMapChanged(Player *player) override {
    if (!player || !player->GetMap() || player->GetMap()->ToInstanceMap())
      return;

    bool hadAura = false;
    if (player->HasAura(FLEX_STAT_SPELL_ID)) {
      LOG_INFO("module.flex", "Flex Stat removed after leaving instance (GUID {})", player->GetGUID().ToString());
      player->RemoveAura(FLEX_STAT_SPELL_ID);
      hadAura = true;
    }
    if (player->HasAura(FLEX_SPELLPOWER_SPELL_ID)) {
      LOG_INFO("module.flex", "Flex SpellPower removed after leaving instance (GUID {})", player->GetGUID().ToString());
      player->RemoveAura(FLEX_SPELLPOWER_SPELL_ID);
      hadAura = true;
    }
    if (hadAura)
      player->UpdateAllStats();

    RemoveFlexSpellPowerFromControlled(player);
  }
};

void Addmod_flexScripts() {
  new flex();
  new flex_player();
}
