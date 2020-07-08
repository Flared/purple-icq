use purple::Plugin;
use purple_sys;
pub struct Connection(*mut purple_sys::PurpleConnection);

impl Connection {
    pub unsafe fn from_raw(ptr: *mut purple_sys::PurpleConnection) -> Self {
        Connection(ptr)
    }

    pub fn get_protocol_plugin(&self) -> Option<Plugin> {
        let plugin_ptr = unsafe { purple_sys::purple_connection_get_prpl(self.0) };
        if plugin_ptr.is_null() {
            None
        } else {
            Some(unsafe { Plugin::from_raw(plugin_ptr) })
        }
    }
}
