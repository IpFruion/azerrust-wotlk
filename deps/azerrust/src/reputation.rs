#[cxx::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("ReputationMgr.h");
        type ReputationMgr;
    }
}
