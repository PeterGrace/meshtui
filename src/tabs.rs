pub(crate) mod about;
mod channels;
pub(crate) mod device_config;
pub(crate) mod messages;
pub(crate) mod modules_config;
pub(crate) mod nodes;

pub use about::AboutTab;
pub use channels::ChannelsTab;
pub use device_config::ConfigTab;
pub use messages::MessagesTab;
pub use modules_config::ModulesConfigTab;
pub use nodes::NodesTab;
