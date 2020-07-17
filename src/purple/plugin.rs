pub struct Plugin(*mut purple_sys::PurplePlugin);

impl Plugin {
    pub unsafe fn from_raw(ptr: *mut purple_sys::PurplePlugin) -> Self {
        Plugin(ptr)
    }

    pub unsafe fn extra<'a, T>(&mut self) -> &'a mut T {
        &mut *((*self.0).extra as *mut T)
    }
}
