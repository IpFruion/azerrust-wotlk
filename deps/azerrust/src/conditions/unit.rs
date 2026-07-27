use std::pin::Pin;

use crate::{conditions::comparison::ComparisonType, ffi};
use enum_dispatch::enum_dispatch;

#[enum_dispatch(UnitConditionTrait)]
pub enum UnitConditionKind {
    Aura(UnitConditionAura),
    Class(UnitConditionClass),
    Race(UnitConditionRace),
    Level(UnitConditionLevel),
    Alive(UnitConditionAlive),
    HpVal(UnitConditionHpVal),
    HpPct(UnitConditionHpPct),
    UnitState(UnitConditionUnitState),
    StandState(UnitConditionStandState),
    InWater(UnitConditionInWater),
    Charmed(UnitConditionCharmed),
    InCombat(UnitConditionInCombat),
    HasAuraType(UnitConditionHasAuraType),
}

#[enum_dispatch]
pub trait UnitConditionTrait {
    fn meets(&self, unit: Pin<&ffi::Unit>) -> bool;
}

pub struct UnitConditionAura {
    spell_id: u32,
    eff_index: u8,
    #[allow(unused)]
    use_target: bool,
}

impl UnitConditionTrait for UnitConditionAura {
    fn meets(&self, unit: Pin<&ffi::Unit>) -> bool {
        crate::ffi::azerrust_unit_has_aura_effect(&unit, self.spell_id, self.eff_index)
    }
}

impl TryFrom<(u32, u32, u32)> for UnitConditionAura {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(UnitConditionAura {
            spell_id: value.0,
            eff_index: value.1 as u8,
            use_target: value.2 != 0,
        })
    }
}

pub struct UnitConditionClass {
    class_mask: u32,
}

impl UnitConditionTrait for UnitConditionClass {
    fn meets(&self, unit: Pin<&ffi::Unit>) -> bool {
        unit.getClassMask() & self.class_mask != 0
    }
}

impl TryFrom<(u32, u32, u32)> for UnitConditionClass {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(UnitConditionClass {
            class_mask: value.0,
        })
    }
}

pub struct UnitConditionRace {
    race_mask: u32,
}

impl UnitConditionTrait for UnitConditionRace {
    fn meets(&self, unit: Pin<&ffi::Unit>) -> bool {
        unit.getRaceMask() & self.race_mask != 0
    }
}

impl TryFrom<(u32, u32, u32)> for UnitConditionRace {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(UnitConditionRace { race_mask: value.0 })
    }
}

pub struct UnitConditionLevel {
    level: u32,
    comparison: ComparisonType,
}

impl UnitConditionTrait for UnitConditionLevel {
    fn meets(&self, unit: Pin<&ffi::Unit>) -> bool {
        self.comparison.compare(unit.GetLevel() as u32, self.level)
    }
}

impl TryFrom<(u32, u32, u32)> for UnitConditionLevel {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(UnitConditionLevel {
            level: value.0,
            comparison: ComparisonType::from_repr(value.1).ok_or(())?,
        })
    }
}

pub struct UnitConditionAlive;

impl UnitConditionTrait for UnitConditionAlive {
    fn meets(&self, unit: Pin<&ffi::Unit>) -> bool {
        unit.IsAlive()
    }
}

impl TryFrom<(u32, u32, u32)> for UnitConditionAlive {
    type Error = ();

    fn try_from(_value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(UnitConditionAlive)
    }
}

pub struct UnitConditionHpVal {
    hp_val: u32,
    comparison: ComparisonType,
}

impl UnitConditionTrait for UnitConditionHpVal {
    fn meets(&self, unit: Pin<&ffi::Unit>) -> bool {
        self.comparison.compare(unit.GetHealth(), self.hp_val)
    }
}

impl TryFrom<(u32, u32, u32)> for UnitConditionHpVal {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(UnitConditionHpVal {
            hp_val: value.0,
            comparison: ComparisonType::from_repr(value.1).ok_or(())?,
        })
    }
}

pub struct UnitConditionHpPct {
    hp_pct: f32,
    comparison: ComparisonType,
}

impl UnitConditionTrait for UnitConditionHpPct {
    fn meets(&self, unit: Pin<&ffi::Unit>) -> bool {
        self.comparison.compare(unit.GetHealthPct(), self.hp_pct)
    }
}

impl TryFrom<(u32, u32, u32)> for UnitConditionHpPct {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(UnitConditionHpPct {
            hp_pct: value.0 as f32,
            comparison: ComparisonType::from_repr(value.1).ok_or(())?,
        })
    }
}

pub struct UnitConditionUnitState {
    state: u32,
}

impl UnitConditionTrait for UnitConditionUnitState {
    fn meets(&self, unit: Pin<&ffi::Unit>) -> bool {
        unit.HasUnitState(self.state)
    }
}

impl TryFrom<(u32, u32, u32)> for UnitConditionUnitState {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(UnitConditionUnitState { state: value.0 })
    }
}

pub enum StandStateKind {
    Exact(u8),
    Standing,
    Sitting,
}

pub struct UnitConditionStandState {
    kind: StandStateKind,
}

impl UnitConditionTrait for UnitConditionStandState {
    fn meets(&self, unit: Pin<&ffi::Unit>) -> bool {
        match &self.kind {
            StandStateKind::Exact(state) => unit.getStandState() == *state,
            StandStateKind::Standing => unit.IsStandState(),
            StandStateKind::Sitting => unit.IsSitState(),
        }
    }
}

impl TryFrom<(u32, u32, u32)> for UnitConditionStandState {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        let kind = match value.0 {
            0 => StandStateKind::Exact(value.1 as u8),
            1 if value.1 == 0 => StandStateKind::Standing,
            1 if value.1 == 1 => StandStateKind::Sitting,
            _ => return Err(()),
        };
        Ok(UnitConditionStandState { kind })
    }
}

pub struct UnitConditionInWater;

impl UnitConditionTrait for UnitConditionInWater {
    fn meets(&self, unit: Pin<&ffi::Unit>) -> bool {
        unit.IsInWater()
    }
}

impl TryFrom<(u32, u32, u32)> for UnitConditionInWater {
    type Error = ();

    fn try_from(_value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(UnitConditionInWater)
    }
}

pub struct UnitConditionCharmed;

impl UnitConditionTrait for UnitConditionCharmed {
    fn meets(&self, unit: Pin<&ffi::Unit>) -> bool {
        unit.IsCharmed()
    }
}

impl TryFrom<(u32, u32, u32)> for UnitConditionCharmed {
    type Error = ();

    fn try_from(_value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(UnitConditionCharmed)
    }
}

pub struct UnitConditionInCombat;

impl UnitConditionTrait for UnitConditionInCombat {
    fn meets(&self, unit: Pin<&ffi::Unit>) -> bool {
        unit.IsInCombat()
    }
}

impl TryFrom<(u32, u32, u32)> for UnitConditionInCombat {
    type Error = ();

    fn try_from(_value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(UnitConditionInCombat)
    }
}

pub struct UnitConditionHasAuraType {
    aura_type: u32,
}

impl UnitConditionTrait for UnitConditionHasAuraType {
    fn meets(&self, unit: Pin<&ffi::Unit>) -> bool {
        crate::ffi::azerrust_unit_has_aura_type(&unit, self.aura_type)
    }
}

impl TryFrom<(u32, u32, u32)> for UnitConditionHasAuraType {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(UnitConditionHasAuraType { aura_type: value.0 })
    }
}
