# Rust Unit Conditions — Design Doc

## Overview

Phase 2 of migrating AzerothCore's Condition system from C++ to Rust. Consolidate all `Unit`-based condition types (13 total, including the 3 already implemented) into a unified `UnitCondition` enum and add a new `conditions/unit.rs` module. This eliminates 3 separate `ConditionKind` variants and simplifies the top-level enum.

## Condition Types

| # | Condition Type | C++ Method | Value1 | Value2 | Value3 |
|---|---|---|---|---|---|---|
| 1 | `CONDITION_AURA` | `Unit::HasAuraEffect(spellId, effIndex)` | spell_id | eff_index | use_target |
| 15 | `CONDITION_CLASS` | `Unit::getClassMask()` | class_mask | — | — |
| 16 | `CONDITION_RACE` | `Unit::getRaceMask()` | race_mask | — | — |
| 21 | `CONDITION_UNIT_STATE` | `Unit::HasUnitState(state)` | state_mask | — | — |
| 27 | `CONDITION_LEVEL` | `Unit::GetLevel()` | level | ComparisonType | — |
| 36 | `CONDITION_ALIVE` | `Unit::IsAlive()` | — | — | — |
| 37 | `CONDITION_HP_VAL` | `Unit::GetHealth()` | hp_val | ComparisonType | — |
| 38 | `CONDITION_HP_PCT` | `Unit::GetHealthPct()` | hp_pct | ComparisonType | — |
| 40 | `CONDITION_IN_WATER` | `Unit::IsInWater()` | — | — | — |
| 42 | `CONDITION_STAND_STATE` | `Unit::getStandState()`, `IsStandState()`, `IsSitState()` | state_type | state | — |
| 44 | `CONDITION_CHARMED` | `Unit::IsCharmed()` | — | — | — |
| 102 | `CONDITION_HAS_AURA_TYPE` | `Unit::HasAuraType(AuraType)` | aura_type | — | — |
| 106 | `CONDITION_UNIT_IN_COMBAT` | `Unit::IsInCombat()` | — | — | — |

## FFI Bridge Expansion

### New `Unit` methods in `src/lib.rs`

```rust
unsafe extern "C++" {
    include!("Unit.h");
    type Unit;

    // existing
    fn GetLevel(&self) -> u8;
    fn getClassMask(&self) -> u32;
    fn getRaceMask(&self) -> u32;

    // new — Phase 2
    fn HasAuraEffect(&self, spellId: u32, effIndex: u8) -> bool;
    fn IsAlive(&self) -> bool;
    fn GetHealth(&self) -> u32;
    fn GetHealthPct(&self) -> f32;
    fn HasUnitState(&self, state: u32) -> bool;
    fn getStandState(&self) -> u8;
    fn IsStandState(&self) -> bool;
    fn IsSitState(&self) -> bool;
    fn IsInWater(&self) -> bool;
    fn IsCharmed(&self) -> bool;
    fn IsInCombat(&self) -> bool;
    fn HasAuraType(&self, auraType: u32) -> bool;
}
```

### Note on `ObjectGuid` in `HasAuraEffect`

`HasAuraEffect(uint32 spellId, uint8 effIndex, ObjectGuid caster = ObjectGuid::Empty)` has a defaulted third parameter. The cxx bridge declares it with only two parameters — CXX generates C++ code that calls `unit->HasAuraEffect(spellId, effIndex)`, and the C++ compiler fills in the `ObjectGuid::Empty` default. No wrapper needed.

## Rust Module Structure

```
src/
  lib.rs                   — cxx bridge + WorldObjectRef
  geometry.rs              — unchanged
  conditions/
    mod.rs                 — Condition, ConditionKind (add Unit variant)
    comparison.rs          — unchanged
    unit.rs                — NEW: UnitCondition enum + per-variant meets()
```

### `conditions/unit.rs`

```rust
pub enum StandStateKind {
    Exact(u8),   // value1=0: compare getStandState() == value2
    Standing,    // value1=1, value2=0: IsStandState()
    Sitting,     // value1=1, value2=1: IsSitState()
}

pub enum UnitCondition {
    Aura { spell_id: u32, eff_index: u8, use_target: bool },
    Class { class_mask: u32 },
    Race { race_mask: u32 },
    Level { level: u32, comparison: ComparisonType },
    Alive,
    HpVal { hp_val: u32, comparison: ComparisonType },
    HpPct { hp_pct: f32, comparison: ComparisonType },
    UnitState { state: u32 },
    StandState(StandStateKind),
    InWater,
    Charmed,
    InCombat,
    HasAuraType { aura_type: u32 },
}

impl UnitCondition {
    pub fn meets(&self, unit: &Unit) -> bool {
        match self {
            UnitCondition::Aura { spell_id, eff_index, use_target: _ } =>
                unit.HasAuraEffect(*spell_id, *eff_index),  // use_target handled in Condition::Meets via ConditionTarget
            UnitCondition::Class { class_mask } =>
                unit.getClassMask() & *class_mask != 0,
            UnitCondition::Race { race_mask } =>
                unit.getRaceMask() & *race_mask != 0,
            UnitCondition::Level { level, comparison } =>
                comparison.compare(unit.GetLevel() as u32, *level),
            UnitCondition::Alive => unit.IsAlive(),
            UnitCondition::HpVal { hp_val, comparison } =>
                comparison.compare(unit.GetHealth(), *hp_val),
            UnitCondition::HpPct { hp_pct, comparison } =>
                comparison.compare(unit.GetHealthPct(), *hp_pct),
            UnitCondition::UnitState { state } =>
                unit.HasUnitState(*state),
            UnitCondition::StandState(kind) => match kind {
                StandStateKind::Exact(state) => unit.getStandState() == *state,
                StandStateKind::Standing => unit.IsStandState(),
                StandStateKind::Sitting => unit.IsSitState(),
            },
            UnitCondition::InWater => unit.IsInWater(),
            UnitCondition::Charmed => unit.IsCharmed(),
            UnitCondition::InCombat => unit.IsInCombat(),
            UnitCondition::HasAuraType { aura_type } =>
                unit.HasAuraType(*aura_type),
        }
    }
}
```

### Simplification of `ConditionKind`

Previously, each condition type group had its own top-level variant in `ConditionKind`. Now all 13 `Unit`-based types are folded into a single `UnitCondition` variant:

In `conditions/mod.rs`:
```rust
pub enum ConditionKind {
    None,
    Unit(UnitCondition),  // 13 condition types, single variant
}
```

The `TryFrom<ffi::Condition>` conversion maps condition types 1, 15, 16, 21, 27, 36, 37, 38, 40, 42, 44, 102, 106 to `ConditionKind::Unit`, replacing the previous separate mappings for 15, 16, 27.

The old `ConditionClass`, `ConditionRace`, and `ConditionLevel` structs are deleted — their logic is now variants within `UnitCondition` in `conditions/unit.rs`.

### Idempotency / C++ fallback

The C++ `Condition::Meets()` already calls `conditions::meets()` first. If Rust returns `0` (false) or `1` (true), that result is used. If Rust returns `-1` (unimplemented), C++ falls through to its own switch. Once all condition types in a category are implemented, the `-1` return is never emitted for those types.

New code is additive — the C++ `switch` branches for Phase 2 condition types remain in place as dead code until they're proven stable.

## Verification

1. **Rust compile**: `cd deps/azerrust && cargo build` succeeds
2. **C++ compile**: CMake build (`docker compose build`) links successfully
3. **Runtime**: worldserver starts, condition checks in-game continue working. The existing CLASS/RACE/LEVEL paths (previously separate `ConditionKind` variants) now route through `UnitCondition` with identical behavior.

## Future Phases (not in scope)

- Phase 1: WorldObject simple conditions (ZoneId, MapId, AreaId, PhaseMask, TypeMask, SpawnMask)
- Phase 3: Player-specific conditions (items, quests, reputation, skills, achievements, etc.)
- Phase 4: Multi-target conditions (RelationTo, ReactionTo, DistanceTo)
- Phase 5: Global/singleton conditions (ActiveEvent, WorldState, RealmAchievement, AI_Data)
