#[cxx::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("Pet.h");
        type Pet;
    }
}
