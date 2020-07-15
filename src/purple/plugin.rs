use purple_sys;

pub struct Plugin(*mut purple_sys::PurplePlugin);

impl Plugin {
    pub unsafe fn from_raw(ptr: *mut purple_sys::PurplePlugin) -> Self {
        Plugin(ptr)
    }

    pub unsafe fn extra<T>(&self) -> &T {
        &*((*self.0).extra as *mut T)
    }
}
