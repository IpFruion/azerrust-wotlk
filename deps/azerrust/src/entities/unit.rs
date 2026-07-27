#[cxx::bridge]
pub mod ffi {
    extern "C++" {
        type Player = crate::entities::player::ffi::Player;
    }
    unsafe extern "C++" {
        include!("Unit.h");
        type Unit;

        fn GetLevel(&self) -> u8;
        fn getClassMask(&self) -> u32;
        fn getRaceMask(&self) -> u32;

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
        unsafe fn IsInPartyWith(&self, other: *const Unit) -> bool;
        unsafe fn IsInRaidWith(&self, other: *const Unit) -> bool;
        unsafe fn IsOnVehicle(&self, other: *const Unit) -> bool;

        // TODO: Address this ASAP!!! This is very spooky but the C++ side requires that the Unit reference is const but the
        // returned player reference is not. DO NOT USE THIS FUNCTION IF YOU CAN HELP IT
        unsafe fn GetCharmerOrOwnerPlayerOrPlayerItself(&self) -> *mut Player;
    }
}
