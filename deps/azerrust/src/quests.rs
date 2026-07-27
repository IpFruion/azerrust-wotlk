use strum::FromRepr;

#[cxx::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("QuestDef.h");
        type Quest;

        fn GetQuestId(&self) -> u32;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
#[repr(u8)]
pub enum QuestStatus {
    None = 0,
    Complete = 1,
    Incomplete = 2,
    Failed = 3,
    Rewarded = 6,
}
