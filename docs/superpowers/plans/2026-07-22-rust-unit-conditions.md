# Rust Unit Conditions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Migrate all Unit-based condition evaluation from C++ to Rust by consolidating 13 condition types into a single `UnitCondition` enum.

**Architecture:** Add ~12 `Unit` FFI methods to the existing cxx bridge. Create a new `conditions/unit.rs` module with `UnitCondition` enum + `meets()`. Simplify `ConditionKind` by folding Class/Race/Level into `UnitCondition`.

**Tech Stack:** Rust `cxx` FFI bridge, Cargo staticlib, CMake build integration via `cargo build --release`.

## Global Constraints

- Existing `conditions::meets()` in C++ returns `-1` for unimplemented types (Rust-side fallback). All 13 types in this plan must return `0` or `1`, eliminating the C++ fallback for Unit conditions.
- CXX bridge methods are declared in `src/lib.rs` with no C++ wrapper needed — C++ default arguments are filled at the call site.
- `ComparisonType` (Eq/High/Low/HighEq/LowEq) already exists in `conditions/comparison.rs` with `from_repr` derive and `compare()` method.
- `TryFrom<ffi::Condition>` conversion lives in `conditions/mod.rs`.

---
### Task 1: Add Unit FFI methods to the cxx bridge

**Files:**
- Modify: `deps/azerrust/src/lib.rs`

**Interfaces:**
- Consumes: Existing `Unit` type in the bridge with `GetLevel`, `getClassMask`, `getRaceMask`
- Produces: Extended `Unit` type with 12 new methods consumable by any Rust code

- [ ] **Step 1: Add new Unit method declarations to the bridge**

Add these declarations inside the existing `unsafe extern "C++"` block for `Unit`:

```rust
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
```

The full block should look like:

```rust
unsafe extern "C++" {
    include!("Unit.h");
    type Unit;

    fn GetLevel(&self) -> u8;
    fn getClassMask(&self) -> u32;
    fn getRaceMask(&self) -> u32;

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

- [ ] **Step 2: Verify Rust compilation**

```bash
cd deps/azerrust && cargo build
```

Expected: Compiles successfully (methods are declared but unused, which is fine).

- [ ] **Step 3: Commit**

```bash
git add deps/azerrust/src/lib.rs
git commit -m "feat(Rust): add Unit FFI methods for condition evaluation"
```

---
### Task 2: Create `conditions/unit.rs` module

**Files:**
- Create: `deps/azerrust/src/conditions/unit.rs`
- Modify: `deps/azerrust/src/conditions/mod.rs` (add `pub mod unit;`)

**Interfaces:**
- Consumes: `ffi::Unit` (via `crate::ffi`), `ComparisonType` (via `crate::conditions::comparison`)
- Produces: `UnitCondition` enum with `meets(&self, unit: &ffi::Unit) -> bool`, `StandStateKind` enum

- [ ] **Step 1: Create unit.rs**

```rust
use crate::conditions::comparison::ComparisonType;
use crate::ffi;

pub enum StandStateKind {
    Exact(u8),
    Standing,
    Sitting,
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
    pub fn meets(&self, unit: &ffi::Unit) -> bool {
        match self {
            UnitCondition::Aura { spell_id, eff_index, use_target: _ } =>
                unit.HasAuraEffect(*spell_id, *eff_index),
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

- [ ] **Step 2: Add `pub mod unit;` to conditions/mod.rs**

Add this line at the top of `conditions/mod.rs`, after the `pub mod comparison;` line:

```rust
pub mod unit;
```

- [ ] **Step 3: Verify Rust compilation**

```bash
cd deps/azerrust && cargo build
```

Expected: Compiles successfully.

- [ ] **Step 4: Commit**

```bash
git add deps/azerrust/src/conditions/unit.rs deps/azerrust/src/conditions/mod.rs
git commit -m "feat(Rust): add UnitCondition enum with 13 condition type variants"
```

---
### Task 3: Refactor `conditions/mod.rs` to use `UnitCondition`

**Files:**
- Modify: `deps/azerrust/src/conditions/mod.rs`

**Interfaces:**
- Consumes: `UnitCondition` (from `super::unit`), `StandStateKind` (from `super::unit`)
- Produces: Simplified `ConditionKind` enum with only `None` and `Unit(UnitCondition)` variants

- [ ] **Step 1: Replace ConditionKind and TryFrom**

Replace the entire contents of `conditions/mod.rs`:

```rust
use crate::conditions::comparison::ComparisonType;
use crate::conditions::unit::{StandStateKind, UnitCondition};
use crate::ffi;
use crate::WorldObjectRef;

pub mod comparison;
pub mod unit;

pub struct Condition {
    pub kind: ConditionKind,
    pub negative: bool,
}

impl Condition {
    pub fn meets(&self, object: WorldObjectRef) -> Result<bool, ()> {
        let res = match &self.kind {
            ConditionKind::None => Ok(true),
            ConditionKind::Unit(unit_cond) => {
                let unit = object.into_unit()?;
                Ok(unit_cond.meets(unit))
            }
        }?;

        Ok(if self.negative { !res } else { res })
    }
}

impl TryFrom<ffi::Condition> for Condition {
    type Error = ();

    fn try_from(value: ffi::Condition) -> Result<Self, Self::Error> {
        let kind = match value.cond_type {
            0 => ConditionKind::None,
            1 => ConditionKind::Unit(UnitCondition::Aura {
                spell_id: value.val1,
                eff_index: value.val2 as u8,
                use_target: value.val3 != 0,
            }),
            15 => ConditionKind::Unit(UnitCondition::Class {
                class_mask: value.val1,
            }),
            16 => ConditionKind::Unit(UnitCondition::Race {
                race_mask: value.val1,
            }),
            21 => ConditionKind::Unit(UnitCondition::UnitState {
                state: value.val1,
            }),
            27 => ConditionKind::Unit(UnitCondition::Level {
                level: value.val1,
                comparison: ComparisonType::from_repr(value.val2).ok_or(())?,
            }),
            36 => ConditionKind::Unit(UnitCondition::Alive),
            37 => ConditionKind::Unit(UnitCondition::HpVal {
                hp_val: value.val1,
                comparison: ComparisonType::from_repr(value.val2).ok_or(())?,
            }),
            38 => ConditionKind::Unit(UnitCondition::HpPct {
                hp_pct: value.val1 as f32,
                comparison: ComparisonType::from_repr(value.val2).ok_or(())?,
            }),
            40 => ConditionKind::Unit(UnitCondition::InWater),
            42 => {
                let kind = match value.val1 {
                    0 => StandStateKind::Exact(value.val2 as u8),
                    1 if value.val2 == 0 => StandStateKind::Standing,
                    1 if value.val2 == 1 => StandStateKind::Sitting,
                    _ => return Err(()),
                };
                ConditionKind::Unit(UnitCondition::StandState(kind))
            }
            44 => ConditionKind::Unit(UnitCondition::Charmed),
            102 => ConditionKind::Unit(UnitCondition::HasAuraType {
                aura_type: value.val1,
            }),
            106 => ConditionKind::Unit(UnitCondition::InCombat),
            _ => {
                return Err(());
            }
        };

        Ok(Condition {
            kind,
            negative: value.negative,
        })
    }
}

pub enum ConditionKind {
    None,
    Unit(UnitCondition),
}
```

- [ ] **Step 2: Verify Rust compilation**

```bash
cd deps/azerrust && cargo build
```

Expected: Compiles successfully.

- [ ] **Step 3: Commit**

```bash
git add deps/azerrust/src/conditions/mod.rs
git commit -m "refactor(Rust): fold Class/Race/Level into UnitCondition, simplify ConditionKind"
```

---
### Task 4: Full build verification

**Files:** None

- [ ] **Step 1: Build the full project with Docker**

```bash
docker compose build
```

Expected: Build succeeds. The `build` stage in `apps/docker/Dockerfile` compiles the Rust crate via CMake's cargo invocation, then links `libazerrust.a` into the worldserver. Watch for:
- `Running Cargo...` step completes without errors
- `azerrust.h` and `azerrust_geometry.h` headers are generated
- C++ compilation of `ConditionMgr.cpp` succeeds (it includes `<azerrust.h>` and calls `conditions::meets()`)
- Final link succeeds

- [ ] **Step 2: Verify runtime behavior (if environment permits)**

If a test server is available, start it and verify condition-based game mechanics work (quest availability, vendor items, gossip menu options, etc.). The Rust code now handles 13 condition types that previously fell through to C++. No behavioral change expected — the meets() logic mirrors the C++ switch exactly.

- [ ] **Step 3: Commit (if any build fixes were needed)**

```bash
git add -A
git commit -m "fix: address build issues from Rust Unit condition migration"
```
