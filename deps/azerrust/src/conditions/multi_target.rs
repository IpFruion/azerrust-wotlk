use enum_dispatch::enum_dispatch;

use crate::{Relation, conditions::comparison::ComparisonType, entities::object::WorldObjectRef};

#[enum_dispatch(MultiTargetConditionTrait)]
pub enum MultiTargetConditionKind {
    Relation(MultiTargetConditionRelationTo),
    Reaction(MultiTargetConditionReactionTo),
    Distance(MultiTargetConditionDistanceTo),
}

impl MultiTargetConditionKind {
    //TODO: move to shared value instead of match
    pub fn target(&self) -> u8 {
        match self {
            MultiTargetConditionKind::Relation(r) => r.target,
            MultiTargetConditionKind::Reaction(r) => r.target,
            MultiTargetConditionKind::Distance(d) => d.target,
        }
    }
}

#[enum_dispatch]
pub trait MultiTargetConditionTrait {
    fn meets(&self, object: WorldObjectRef, target: WorldObjectRef) -> Result<bool, ()>;
}

pub struct MultiTargetConditionRelationTo {
    target: u8,
    relation_type: Relation,
}

impl MultiTargetConditionTrait for MultiTargetConditionRelationTo {
    fn meets(&self, mut object: WorldObjectRef, mut target: WorldObjectRef) -> Result<bool, ()> {
        let unit_a = object.as_unit()?;
        let unit_b = target.as_unit()?;
        Ok(match self.relation_type {
            Relation::Equal => std::ptr::eq(&*unit_a as *const _, &*unit_b as *const _),
            Relation::InParty => unsafe { unit_a.IsInPartyWith(&*unit_b as *const _) },
            Relation::InRaidOrParty => unsafe { unit_a.IsInRaidWith(&*unit_b as *const _) },
            Relation::OwnedBy => crate::ffi::azerrust_unit_has_owner(&unit_a, &unit_b),
            Relation::PassengerOf => unsafe { unit_a.IsOnVehicle(&*unit_b as *const _) },
            Relation::CreatedBy => crate::ffi::azerrust_unit_has_creator(&unit_a, &unit_b),
        })
    }
}

impl TryFrom<(u32, u32, u32)> for MultiTargetConditionRelationTo {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(MultiTargetConditionRelationTo {
            target: value.0 as u8,
            relation_type: Relation::from_repr(value.1).ok_or(())?,
        })
    }
}

pub struct MultiTargetConditionReactionTo {
    target: u8,
    rank_mask: u32,
}

impl MultiTargetConditionTrait for MultiTargetConditionReactionTo {
    fn meets(&self, object: WorldObjectRef, target: WorldObjectRef) -> Result<bool, ()> {
        let reaction = crate::ffi::azerrust_worldobject_check_reaction(&object, &target);
        if reaction < 0 {
            return Ok(false);
        }
        Ok((1 << reaction as u32) & self.rank_mask != 0)
    }
}

impl TryFrom<(u32, u32, u32)> for MultiTargetConditionReactionTo {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(MultiTargetConditionReactionTo {
            target: value.0 as u8,
            rank_mask: value.1,
        })
    }
}

pub struct MultiTargetConditionDistanceTo {
    target: u8,
    distance: f32,
    comparison: ComparisonType,
}

impl MultiTargetConditionTrait for MultiTargetConditionDistanceTo {
    fn meets(&self, object: WorldObjectRef, target: WorldObjectRef) -> Result<bool, ()> {
        let dist = crate::ffi::azerrust_worldobject_get_distance(&object, &target);
        Ok(self.comparison.compare(dist, self.distance))
    }
}

impl TryFrom<(u32, u32, u32)> for MultiTargetConditionDistanceTo {
    type Error = ();

    fn try_from(value: (u32, u32, u32)) -> Result<Self, Self::Error> {
        Ok(MultiTargetConditionDistanceTo {
            target: value.0 as u8,
            distance: f32::from_bits(value.2),
            comparison: ComparisonType::from_repr(value.1).ok_or(())?,
        })
    }
}
