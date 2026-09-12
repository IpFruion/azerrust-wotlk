/*
 * This file is part of the AzerothCore Project. See AUTHORS file for Copyright information
 *
 * This program is free software; you can redistribute it and/or modify it
 * under the terms of the GNU General Public License as published by the
 * Free Software Foundation; either version 2 of the License, or (at your
 * option) any later version.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
 * FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for
 * more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program. If not, see <http://www.gnu.org/licenses/>.
 */

#include "AllCreatureScript.h"
#include "AllMapScript.h"
#include "Chat.h"
#include "CommandScript.h"
#include "Config.h"
#include "Creature.h"
#include "DBCEnums.h"
#include "Log.h"
#include "Map.h"
#include "Player.h"
#include "PlayerScript.h"
#include "SpellAuras.h"
#include "SpellAuraEffects.h"
#include "SpellDefines.h"
#include "SpellMgr.h"
#include "Unit.h"
#include "Vehicle.h"
#include "World.h"

#include <algorithm>
#include <array>

static uint32 constexpr FLEX_STAT_SPELL_ID = 100103;
static uint32 constexpr FLEX_SPELLPOWER_SPELL_ID = 100104;
static uint32 constexpr FLEX_PHYSICAL_SPELL_ID = 100106;
static uint32 constexpr FLEX_HEALING_SPELL_ID = 100107;
static uint32 constexpr FLEX_MAX_STACKS = 40;

static constexpr std::array<uint32, 4> FLEX_SPELL_IDS = {
    FLEX_STAT_SPELL_ID,
    FLEX_SPELLPOWER_SPELL_ID,
    FLEX_PHYSICAL_SPELL_ID,
    FLEX_HEALING_SPELL_ID
};

/// Persisted per-character role, declared with ".flex role". Values are stored in
/// character_settings, so they must stay stable across releases.
enum class FlexRole : uint8
{
    None = 0,
    Tank = 1,
    Healer = 2,
    Damage = 3
};

static std::string const FLEX_SETTINGS_SOURCE = "mod_flex";
static uint32 constexpr FLEX_SETTING_ROLE = 0;

/// Custom RBAC permission granted to the player role by the module's auth SQL update.
static uint32 constexpr FLEX_RBAC_PERM_COMMAND_ROLE = 946;

struct FlexRates
{
    uint32 statPercent = 0;
    uint32 spellPowerPercent = 0;
    uint32 physicalPercent = 0;
    uint32 tankStatPercent = 0;
    uint32 healingPercent = 0;
    bool valid = false;
};

struct FlexConfig
{
    uint32 dungeonNormalStat = 0;
    uint32 dungeonHeroicStat = 0;
    uint32 raidNormalStat = 0;
    uint32 raidHeroicStat = 0;
    uint32 raidMythicStat = 0;
    uint32 dungeonNormalSpellPower = 0;
    uint32 dungeonHeroicSpellPower = 0;
    uint32 raidNormalSpellPower = 0;
    uint32 raidHeroicSpellPower = 0;
    uint32 raidMythicSpellPower = 0;
    uint32 dungeonNormalPhysical = 0;
    uint32 dungeonHeroicPhysical = 0;
    uint32 raidNormalPhysical = 0;
    uint32 raidHeroicPhysical = 0;
    uint32 raidMythicPhysical = 0;
    uint32 dungeonNormalTankStat = 0;
    uint32 dungeonHeroicTankStat = 0;
    uint32 raidNormalTankStat = 0;
    uint32 raidHeroicTankStat = 0;
    uint32 raidMythicTankStat = 0;
    uint32 dungeonNormalHealing = 0;
    uint32 dungeonHeroicHealing = 0;
    uint32 raidNormalHealing = 0;
    uint32 raidHeroicHealing = 0;
    uint32 raidMythicHealing = 0;
};

static FlexConfig g_flexConfig;

static uint32 ClampPercent(int32 raw)
{
    return raw < 0 ? 0u : static_cast<uint32>(raw);
}

static uint32 ReadPercent(char const* key)
{
    return ClampPercent(sConfigMgr->GetOption<int32>(key, 0));
}

static void LoadFlexConfig()
{
    g_flexConfig.dungeonNormalStat = ReadPercent("Flex.Dungeon.Normal.Stat.Percent");
    g_flexConfig.dungeonHeroicStat = ReadPercent("Flex.Dungeon.Heroic.Stat.Percent");
    g_flexConfig.raidNormalStat = ReadPercent("Flex.Raid.Normal.Stat.Percent");
    g_flexConfig.raidHeroicStat = ReadPercent("Flex.Raid.Heroic.Stat.Percent");
    g_flexConfig.raidMythicStat = ReadPercent("Flex.Raid.Mythic.Stat.Percent");

    g_flexConfig.dungeonNormalSpellPower = ReadPercent("Flex.Dungeon.Normal.SpellPower.Percent");
    g_flexConfig.dungeonHeroicSpellPower = ReadPercent("Flex.Dungeon.Heroic.SpellPower.Percent");
    g_flexConfig.raidNormalSpellPower = ReadPercent("Flex.Raid.Normal.SpellPower.Percent");
    g_flexConfig.raidHeroicSpellPower = ReadPercent("Flex.Raid.Heroic.SpellPower.Percent");
    g_flexConfig.raidMythicSpellPower = ReadPercent("Flex.Raid.Mythic.SpellPower.Percent");

    g_flexConfig.dungeonNormalPhysical = ReadPercent("Flex.Dungeon.Normal.Physical.Percent");
    g_flexConfig.dungeonHeroicPhysical = ReadPercent("Flex.Dungeon.Heroic.Physical.Percent");
    g_flexConfig.raidNormalPhysical = ReadPercent("Flex.Raid.Normal.Physical.Percent");
    g_flexConfig.raidHeroicPhysical = ReadPercent("Flex.Raid.Heroic.Physical.Percent");
    g_flexConfig.raidMythicPhysical = ReadPercent("Flex.Raid.Mythic.Physical.Percent");

    g_flexConfig.dungeonNormalTankStat = ReadPercent("Flex.Dungeon.Normal.TankStat.Percent");
    g_flexConfig.dungeonHeroicTankStat = ReadPercent("Flex.Dungeon.Heroic.TankStat.Percent");
    g_flexConfig.raidNormalTankStat = ReadPercent("Flex.Raid.Normal.TankStat.Percent");
    g_flexConfig.raidHeroicTankStat = ReadPercent("Flex.Raid.Heroic.TankStat.Percent");
    g_flexConfig.raidMythicTankStat = ReadPercent("Flex.Raid.Mythic.TankStat.Percent");

    g_flexConfig.dungeonNormalHealing = ReadPercent("Flex.Dungeon.Normal.Healing.Percent");
    g_flexConfig.dungeonHeroicHealing = ReadPercent("Flex.Dungeon.Heroic.Healing.Percent");
    g_flexConfig.raidNormalHealing = ReadPercent("Flex.Raid.Normal.Healing.Percent");
    g_flexConfig.raidHeroicHealing = ReadPercent("Flex.Raid.Heroic.Healing.Percent");
    g_flexConfig.raidMythicHealing = ReadPercent("Flex.Raid.Mythic.Healing.Percent");
}

static FlexRates SelectRates(InstanceMap const* instance)
{
    FlexRates rates;
    if (!instance)
        return rates;

    Difficulty const difficulty = instance->GetDifficulty();
    if (instance->IsNonRaidDungeon())
    {
        if (difficulty == DUNGEON_DIFFICULTY_NORMAL)
        {
            rates = { g_flexConfig.dungeonNormalStat, g_flexConfig.dungeonNormalSpellPower,
                g_flexConfig.dungeonNormalPhysical, g_flexConfig.dungeonNormalTankStat,
                g_flexConfig.dungeonNormalHealing, true };
        }
        else if (difficulty == DUNGEON_DIFFICULTY_HEROIC)
        {
            rates = { g_flexConfig.dungeonHeroicStat, g_flexConfig.dungeonHeroicSpellPower,
                g_flexConfig.dungeonHeroicPhysical, g_flexConfig.dungeonHeroicTankStat,
                g_flexConfig.dungeonHeroicHealing, true };
        }
        return rates;
    }

    if (instance->IsRaid())
    {
        if (difficulty <= RAID_DIFFICULTY_25MAN_NORMAL)
        {
            rates = { g_flexConfig.raidNormalStat, g_flexConfig.raidNormalSpellPower,
                g_flexConfig.raidNormalPhysical, g_flexConfig.raidNormalTankStat,
                g_flexConfig.raidNormalHealing, true };
        }
        else if (difficulty <= RAID_DIFFICULTY_25MAN_HEROIC)
        {
            rates = { g_flexConfig.raidHeroicStat, g_flexConfig.raidHeroicSpellPower,
                g_flexConfig.raidHeroicPhysical, g_flexConfig.raidHeroicTankStat,
                g_flexConfig.raidHeroicHealing, true };
        }
        else
        {
            rates = { g_flexConfig.raidMythicStat, g_flexConfig.raidMythicSpellPower,
                g_flexConfig.raidMythicPhysical, g_flexConfig.raidMythicTankStat,
                g_flexConfig.raidMythicHealing, true };
        }
    }

    return rates;
}

static uint32 CountActiveNonGmPlayers(InstanceMap const* instance)
{
    uint32 count = 0;
    for (Map::PlayerList::const_iterator itr = instance->GetPlayers().begin();
         itr != instance->GetPlayers().end(); ++itr)
    {
        Player* plr = itr->GetSource();
        if (plr && !plr->IsGameMaster())
            ++count;
    }
    return count;
}

static uint32 CountMissingPlayers(InstanceMap const* instance)
{
    uint32 const maxPlayers = instance->GetMaxPlayers();
    uint32 const activePlayers = CountActiveNonGmPlayers(instance);
    return maxPlayers > activePlayers ? maxPlayers - activePlayers : 0;
}

static FlexRole GetFlexRole(Player* player)
{
    switch (player->GetPlayerSetting(FLEX_SETTINGS_SOURCE, FLEX_SETTING_ROLE).value)
    {
        case uint32(FlexRole::Tank):
            return FlexRole::Tank;
        case uint32(FlexRole::Healer):
            return FlexRole::Healer;
        case uint32(FlexRole::Damage):
            return FlexRole::Damage;
        default:
            return FlexRole::None;
    }
}

static char const* GetFlexRoleName(FlexRole role)
{
    switch (role)
    {
        case FlexRole::Tank:
            return "tank";
        case FlexRole::Healer:
            return "healer";
        case FlexRole::Damage:
            return "damage";
        default:
            return "none";
    }
}

static void RemoveFlexAuras(Unit* unit)
{
    if (!unit)
        return;

    for (uint32 spellId : FLEX_SPELL_IDS)
        if (unit->HasAura(spellId))
            unit->RemoveAura(spellId);
}

static void SetFlexAura(Unit* unit, uint32 spellId, uint32 percent, uint32 stacks)
{
    if (!unit)
        return;

    if (percent == 0 || stacks == 0)
    {
        if (unit->HasAura(spellId))
            unit->RemoveAura(spellId);
        return;
    }

    uint8 const stackAmount = static_cast<uint8>(std::min(stacks, FLEX_MAX_STACKS));
    int32 const percentAmount = static_cast<int32>(percent);

    if (Aura* aura = unit->GetAura(spellId))
    {
        if (AuraEffect const* effect = aura->GetEffect(EFFECT_0))
        {
            uint8 const currentStacks = aura->GetStackAmount();
            int32 const perStack = currentStacks ? effect->GetAmount() / int32(currentStacks) : 0;
            if (perStack == percentAmount)
            {
                if (currentStacks != stackAmount)
                    aura->SetStackAmount(stackAmount);
                return;
            }
        }
        unit->RemoveAura(spellId);
    }

    CustomSpellValues values;
    values.AddSpellMod(SPELLVALUE_BASE_POINT0, percentAmount);
    if (stackAmount > 1)
        values.AddSpellMod(SPELLVALUE_AURA_STACK, stackAmount);

    SpellCastResult const result = unit->CastCustomSpell(spellId, values, unit, TRIGGERED_FULL_MASK);
    if (result != SPELL_CAST_OK)
        LOG_ERROR("module.flex", "Failed to cast spell {} on unit (GUID {}): result {}",
            spellId, unit->GetGUID().ToString(), result);
}

static void ApplyRoleAuras(Unit* unit, FlexRole role, FlexRates const& rates, uint32 stacks)
{
    uint32 const magicPercent = role == FlexRole::Damage ? rates.spellPowerPercent : 0;
    uint32 const physicalPercent = role == FlexRole::Damage ? rates.physicalPercent : 0;
    uint32 const tankStatPercent = role == FlexRole::Tank ? rates.tankStatPercent : 0;
    uint32 const healingPercent = role == FlexRole::Healer ? rates.healingPercent : 0;

    SetFlexAura(unit, FLEX_SPELLPOWER_SPELL_ID, magicPercent, stacks);
    SetFlexAura(unit, FLEX_PHYSICAL_SPELL_ID, physicalPercent, stacks);
    SetFlexAura(unit, FLEX_STAT_SPELL_ID, tankStatPercent, stacks);
    SetFlexAura(unit, FLEX_HEALING_SPELL_ID, healingPercent, stacks);
}

static void ForEachControlledUnit(Player* player, void (*callback)(Unit*))
{
    if (!player || !callback)
        return;

    for (Unit* controlled : player->m_Controlled)
        if (controlled && controlled != player)
            callback(controlled);

    if (Vehicle* vehicle = player->GetVehicle())
        if (Unit* base = vehicle->GetBase())
            if (base != player)
                callback(base);
}

static void ApplyFlexToControlled(Player* player, FlexRates const& rates, uint32 stacks)
{
    if (!player)
        return;

    FlexRole const role = GetFlexRole(player);
    for (Unit* controlled : player->m_Controlled)
    {
        if (!controlled || controlled == player)
            continue;
        ApplyRoleAuras(controlled, role, rates, stacks);
    }

    if (Vehicle* vehicle = player->GetVehicle())
        if (Unit* base = vehicle->GetBase())
            if (base != player)
                ApplyRoleAuras(base, role, rates, stacks);
}

static void ApplyFlexToPlayer(Player* player, FlexRates const& rates, uint32 stacks)
{
    if (!player)
        return;

    if (player->IsGameMaster() || !rates.valid || !player->IsAlive())
    {
        RemoveFlexAuras(player);
        ForEachControlledUnit(player, &RemoveFlexAuras);
        return;
    }

    SetFlexAura(player, FLEX_STAT_SPELL_ID, rates.statPercent, stacks);
    ApplyRoleAuras(player, GetFlexRole(player), rates, stacks);
    ApplyFlexToControlled(player, rates, stacks);
}

static void SyncPlayerFlex(Player* player)
{
    if (!player)
        return;

    Map* map = player->GetMap();
    InstanceMap* instance = map ? map->ToInstanceMap() : nullptr;
    if (!instance)
    {
        RemoveFlexAuras(player);
        ForEachControlledUnit(player, &RemoveFlexAuras);
        return;
    }

    ApplyFlexToPlayer(player, SelectRates(instance), CountMissingPlayers(instance));
}

static void RecalculateBuffs(InstanceMap* instance)
{
    if (!instance)
        return;

    FlexRates const rates = SelectRates(instance);
    if (!rates.valid)
    {
        for (Map::PlayerList::const_iterator itr = instance->GetPlayers().begin();
             itr != instance->GetPlayers().end(); ++itr)
        {
            if (Player* plr = itr->GetSource())
            {
                RemoveFlexAuras(plr);
                ForEachControlledUnit(plr, &RemoveFlexAuras);
            }
        }
        LOG_INFO("module.flex", "Flex ignored unsupported map {} difficulty {}",
            instance->GetId(), instance->GetDifficulty());
        return;
    }

    uint32 const stacks = CountMissingPlayers(instance);
    uint32 const activePlayers = CountActiveNonGmPlayers(instance);
    LOG_INFO("module.flex",
        "Flex stacks for map {} instance {}: {} eligible players / {}, {} stacks (stat {}%, magic {}%, physical {}%, tank stat {}%, healing {}%)",
        instance->GetId(), instance->GetInstanceId(), activePlayers, instance->GetMaxPlayers(), stacks,
        rates.statPercent, rates.spellPowerPercent, rates.physicalPercent, rates.tankStatPercent, rates.healingPercent);

    for (Map::PlayerList::const_iterator itr = instance->GetPlayers().begin();
         itr != instance->GetPlayers().end(); ++itr)
        if (Player* plr = itr->GetSource())
            ApplyFlexToPlayer(plr, rates, stacks);
}

class flex : public AllMapScript
{
public:
    flex() : AllMapScript("flex")
    {
        LoadFlexConfig();

        LOG_INFO("module.flex",
            "Flex loaded with stat rates: dungeon {}%, heroic dungeon {}%, raid {}%, heroic raid {}%, mythic raid {}%; "
            "magic rates: dungeon {}%, heroic dungeon {}%, raid {}%, heroic raid {}%, mythic raid {}%; "
            "physical rates: dungeon {}%, heroic dungeon {}%, raid {}%, heroic raid {}%, mythic raid {}%; "
            "tank stat rates: dungeon {}%, heroic dungeon {}%, raid {}%, heroic raid {}%, mythic raid {}%; "
            "healing rates: dungeon {}%, heroic dungeon {}%, raid {}%, heroic raid {}%, mythic raid {}%",
            g_flexConfig.dungeonNormalStat, g_flexConfig.dungeonHeroicStat, g_flexConfig.raidNormalStat,
            g_flexConfig.raidHeroicStat, g_flexConfig.raidMythicStat,
            g_flexConfig.dungeonNormalSpellPower, g_flexConfig.dungeonHeroicSpellPower, g_flexConfig.raidNormalSpellPower,
            g_flexConfig.raidHeroicSpellPower, g_flexConfig.raidMythicSpellPower,
            g_flexConfig.dungeonNormalPhysical, g_flexConfig.dungeonHeroicPhysical, g_flexConfig.raidNormalPhysical,
            g_flexConfig.raidHeroicPhysical, g_flexConfig.raidMythicPhysical,
            g_flexConfig.dungeonNormalTankStat, g_flexConfig.dungeonHeroicTankStat, g_flexConfig.raidNormalTankStat,
            g_flexConfig.raidHeroicTankStat, g_flexConfig.raidMythicTankStat,
            g_flexConfig.dungeonNormalHealing, g_flexConfig.dungeonHeroicHealing, g_flexConfig.raidNormalHealing,
            g_flexConfig.raidHeroicHealing, g_flexConfig.raidMythicHealing);

        for (uint32 spellId : FLEX_SPELL_IDS)
            if (!sSpellMgr->GetSpellInfo(spellId))
                LOG_ERROR("module.flex",
                    "Flex spell {} was not found; the pending world SQL update may not be imported", spellId);

        // PlayerSettings check removed — CONFIG_PLAYER_SETTINGS_ENABLED may not be registered in all forks.
    }

    void OnPlayerEnterAll(Map* map, Player* player) override
    {
        if (!map || !player)
            return;

        InstanceMap* instance = map->ToInstanceMap();
        if (!instance)
            return;

        RecalculateBuffs(instance);
    }

    void OnPlayerLeaveAll(Map* map, Player* player) override
    {
        if (!map || !player)
            return;

        InstanceMap* instance = map->ToInstanceMap();
        if (!instance)
            return;

        RemoveFlexAuras(player);
        ForEachControlledUnit(player, &RemoveFlexAuras);
        RecalculateBuffs(instance);
    }
};

class flex_player : public PlayerScript
{
public:
    flex_player() : PlayerScript("flex_player", {
        PLAYERHOOK_ON_PLAYER_RESURRECT,
        PLAYERHOOK_ON_MAP_CHANGED
    }) { }

    void OnPlayerResurrect(Player* player, float /*restore_percent*/, bool& /*applySickness*/) override
    {
        SyncPlayerFlex(player);
    }

    void OnPlayerMapChanged(Player* player) override
    {
        SyncPlayerFlex(player);
    }
};

class flex_creature : public AllCreatureScript
{
public:
    flex_creature() : AllCreatureScript("flex_creature") { }

    void OnCreatureAddWorld(Creature* creature) override
    {
        if (!creature)
            return;

        Unit* owner = creature->GetOwner();
        if (!owner || !owner->IsPlayer())
            return;

        Player* player = owner->ToPlayer();
        if (!player || player->IsGameMaster() || !player->IsAlive())
            return;

        Map* map = creature->GetMap();
        InstanceMap* instance = map ? map->ToInstanceMap() : nullptr;
        if (!instance)
            return;

        FlexRates const rates = SelectRates(instance);
        if (!rates.valid)
            return;

        ApplyRoleAuras(creature, GetFlexRole(player), rates, CountMissingPlayers(instance));
    }
};

using namespace Acore::ChatCommands;

class flex_commandscript : public CommandScript
{
public:
    flex_commandscript() : CommandScript("flex_commandscript") { }

    ChatCommandTable GetCommands() const override
    {
        static ChatCommandTable flexCommandTable =
        {
            { "role", HandleFlexRoleCommand, FLEX_RBAC_PERM_COMMAND_ROLE, Console::No }
        };

        static ChatCommandTable commandTable =
        {
            { "flex", flexCommandTable }
        };

        return commandTable;
    }

private:
    static bool HandleFlexRoleCommand(ChatHandler* handler, Optional<std::string_view> roleArg)
    {
        Player* player = handler->GetPlayer();
        if (!player)
            return false;

        if (!roleArg)
        {
            handler->PSendSysMessage("Your Flex role is: {}. Use .flex role tank|healer|damage|none",
                GetFlexRoleName(GetFlexRole(player)));
            return true;
        }

        FlexRole role;
        if (*roleArg == "tank")
            role = FlexRole::Tank;
        else if (*roleArg == "healer" || *roleArg == "heal")
            role = FlexRole::Healer;
        else if (*roleArg == "damage" || *roleArg == "dps")
            role = FlexRole::Damage;
        else if (*roleArg == "none" || *roleArg == "clear")
            role = FlexRole::None;
        else
        {
            handler->SendErrorMessage("Unknown role '{}'. Use tank, healer, damage or none.", *roleArg);
            return false;
        }

        player->UpdatePlayerSetting(FLEX_SETTINGS_SOURCE, FLEX_SETTING_ROLE, uint32(role));
        SyncPlayerFlex(player);

        handler->PSendSysMessage("Flex role set to: {}", GetFlexRoleName(role));
        if (role == FlexRole::None)
            handler->PSendSysMessage("You will only receive the Flex stat bonus.");

        return true;
    }
};

void Addmod_flexScripts()
{
    new flex();
    new flex_player();
    new flex_creature();
    new flex_commandscript();
}
