use std::{ops::Deref, pin::Pin};

#[cxx::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("Creature.h");
        type Creature;
        type CreatureTemplate;

        unsafe fn GetCreatureTemplate(self: &Creature) -> *const CreatureTemplate;
        unsafe fn AI(self: &Creature) -> *mut CreatureAI;
        fn GetSpawnId(self: &Creature) -> u32;
    }

    unsafe extern "C++" {
        include!("CreatureAI.h");
        type CreatureAI;

        fn GetData(&self, value: u32) -> u32;
    }
}

pub struct CreatureRef<'a>(pub Pin<&'a mut ffi::Creature>);

impl<'a> Deref for CreatureRef<'a> {
    type Target = Pin<&'a mut ffi::Creature>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> CreatureRef<'a> {
    pub fn creature_template(&self) -> Result<&ffi::CreatureTemplate, ()> {
        unsafe { self.0.GetCreatureTemplate().as_ref() }.ok_or(())
    }

    pub fn ai(&mut self) -> Option<Pin<&'a mut ffi::CreatureAI>> {
        Some(unsafe { Pin::new_unchecked(self.0.AI().as_mut()?) })
    }
}
