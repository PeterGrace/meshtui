use crate::packet_handler::MessageEnvelope;
use meshtastic::protobufs::{FromRadio, ToRadio};

#[derive(Debug)]
pub enum IPCMessage {
    FromRadio(FromRadio),
    ToRadio(ToRadio),
    SendMessage(MessageEnvelope),
    ExitingThread,
}
