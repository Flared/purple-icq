use super::handlers::entrypoints;
use crate::purple::PrplPlugin;
use log::info;
use std::ffi::CString;
use std::marker::PhantomData;
use std::os::raw::c_void;

#[derive(Default)]
pub struct RegisterContext<P> {
    info: Box<purple_sys::PurplePluginInfo>,
    extra_info: Box<purple_sys::PurplePluginProtocolInfo>,
    _phantom_marker: PhantomData<P>,
}

impl<P> RegisterContext<P> {
    pub fn new() -> Self {
        RegisterContext {
            info: Box::new(purple_sys::PurplePluginInfo::default()),
            extra_info: Box::new(purple_sys::PurplePluginProtocolInfo::default()),
            _phantom_marker: PhantomData,
        }
    }
    pub fn into_raw(mut self) -> *mut purple_sys::PurplePluginInfo {
        self.extra_info.roomlist_get_list = Some(entrypoints::roomlist_get_list_handler);

        self.info.extra_info = Box::into_raw(self.extra_info) as *mut c_void;

        Box::into_raw(self.info)
    }

    pub fn with_info(mut self, info: PrplInfo) -> Self {
        self.info.id = CString::new(info.id).unwrap().into_raw();
        self.info.name = CString::new(info.name).unwrap().into_raw();
        self.info.version = CString::new(info.version).unwrap().into_raw();
        self.info.summary = CString::new(info.summary).unwrap().into_raw();
        self.info.description = CString::new(info.description).unwrap().into_raw();
        self.info.author = CString::new(info.author).unwrap().into_raw();
        self.info.homepage = CString::new(info.homepage).unwrap().into_raw();
        self.info.actions = Some(entrypoints::actions);
        self
    }
}

pub struct PrplPluginLoader<P: PrplPlugin>(*mut purple_sys::PurplePlugin, PhantomData<P>);

impl<P: PrplPlugin> PrplPluginLoader<P> {
    pub unsafe fn from_raw(ptr: *mut purple_sys::PurplePlugin) -> Self {
        Self(ptr, PhantomData)
    }

    pub fn init(&self) -> i32 {
        let prpl_plugin = Box::new(P::new());
        let register_context: RegisterContext<P::Plugin> = RegisterContext::new();
        let register_context = prpl_plugin.register(register_context);

        // Unsafe required to dereference the pointers and call
        // purple_plugin_register. Safe otherwise.
        unsafe {
            (*self.0).info = register_context.into_raw();
            (*self.0).extra = Box::into_raw(prpl_plugin) as *mut c_void;

            info!("Registering");
            purple_sys::purple_plugin_register(self.0)
        }
    }
}

pub struct PrplInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub summary: String,
    pub description: String,
    pub author: String,
    pub homepage: String,
}

impl Default for PrplInfo {
    fn default() -> Self {
        PrplInfo {
            id: "".into(),
            name: "".into(),
            version: "".into(),
            summary: "".into(),
            description: "".into(),
            author: "".into(),
            homepage: "".into(),
        }
    }
}

macro_rules! impl_handler_builder {
    ($($f:ident => $t:ident)*) => ($(
        paste::item! {
            impl<T: crate::purple::handlers::traits::[<$t>]> RegisterContext<T> {
                #[allow(dead_code)]
                pub fn [<enable_$f>](mut self) -> Self {
                    self.info.[<$f>] = Some(crate::purple::handlers::entrypoints::[<$f>]::<T>);
                    self
                }
            }
        }
    )*)
}

macro_rules! impl_extra_handler_builder {
    ($($f:ident => $t:ident)*) => ($(
        paste::item! {
            impl<T: crate::purple::handlers::traits::[<$t>]> RegisterContext<T> {
                #[allow(dead_code)]
                pub fn [<enable_$f>](mut self) -> Self {
                    self.extra_info.[<$f>] = Some(crate::purple::handlers::entrypoints::[<$f>]::<T>);
                    self
                }
            }
        }
    )*)
}

impl_handler_builder! {
    load => LoadHandler
}

impl_extra_handler_builder! {
    login => LoginHandler
    chat_info => ChatInfoHandler
    chat_info_defaults => ChatInfoDefaultsHandler
    close => CloseHandler
    status_types => StatusTypeHandler
    list_icon => ListIconHandler
    join_chat => JoinChatHandler
    chat_leave => ChatLeaveHandler
    convo_closed => ConvoClosedHandler
    get_chat_name => GetChatNameHandler
}
