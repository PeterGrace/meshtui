pub(crate) mod about;
pub(crate) mod device_config;
pub(crate) mod messages;
pub(crate) mod nodes;
pub(crate) mod modules_config;
mod channels;

pub use about::AboutTab;
pub use device_config::ConfigTab;
pub use messages::MessagesTab;
pub use nodes::NodesTab;
pub use modules_config::ModulesConfigTab;
pub use channels::ChannelsTab;