pub use self::account::Account;
pub use self::connection::Connection;
pub use self::handlers::traits::*;
pub use self::loader::{PrplInfo, PrplPluginLoader, RegisterContext};
pub use self::plugin::Plugin;
pub use self::status_type::{PurpleStatusPrimitive, StatusType};
mod account;
mod connection;
mod handlers;
mod loader;
mod plugin;
mod status_type;

pub trait PrplPlugin {
    type Plugin;
    fn new() -> Self;
    fn register(&self, context: RegisterContext<Self::Plugin>) -> RegisterContext<Self::Plugin>;
}

macro_rules! purple_prpl_plugin {
    ($plugin:ty) => {
        /// # Safety
        /// This function is the plugin entrypoints and should not be called manually.
        #[no_mangle]
        pub unsafe extern "C" fn purple_init_plugin(
            plugin_ptr: *mut purple_sys::PurplePlugin,
        ) -> i32 {
            // Safe as long as called from libpurple. Should be the
            // case since this function is called by libpurple.
            let plugin = purple::PrplPluginLoader::<$plugin>::from_raw(plugin_ptr);
            plugin.init()
        }
    };
}
