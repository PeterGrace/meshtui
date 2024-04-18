use meshtastic::protobufs::{FromRadio, ToRadio};

#[derive(Debug)]
pub enum IPCMessage {
    FromRadio(FromRadio),
    ToRadio(ToRadio),
    ExitingThread,
}
