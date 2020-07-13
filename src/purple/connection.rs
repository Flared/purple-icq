use super::Plugin;
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

    //    pub fn request_input(
    //        &self,
    //        title: Option<&str>,
    //        primary: Option<&str>,
    //        secondary: Option<&str>,
    //        default_value: Option<&str>,
    //        multiline: bool,
    //        masked: bool,
    //        hint: Option<&str>,
    //        ok_text: &str,
    //        cancel_text: &str,
    //    ) {
    //        let c_title = CString::new(title).unwrap().into_raw();
    //        let c_primary = CString::new(primary).unwrap().into_raw();
    //        let c_secondary = CString::new(secondary).unwrap().into_raw();
    //        let c_default_value = CString::new(default_value).unwrap().into_raw();
    //    }
}
