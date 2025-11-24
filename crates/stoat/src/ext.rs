use stoat_models::v0::Channel;

pub trait ChannelExt {
    fn server(&self) -> Option<&str>;
}

impl ChannelExt for Channel {
    fn server(&self) -> Option<&str> {
        match self {
            Channel::TextChannel { server, .. } => Some(server),
            _ => None
        }
    }
}
