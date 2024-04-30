use crate::app::Connection;
use crate::ipc::IPCMessage;
use anyhow::{bail, Result};




use meshtastic::packet::{PacketRouter};

use meshtastic::protobufs::{FromRadio, MeshPacket};
use meshtastic::types::NodeId;
use meshtastic::{api::StreamApi, utils};
use strum::Display;
use thiserror::Error;

#[derive(Display, Clone, Debug, Error)]
pub enum DeviceUpdateError {
    PacketNotSupported(String),
    RadioMessageNotSupported(String),
    DecodeFailure(String),
    GeneralFailure(String),
    EventDispatchFailure(String),
    NotificationDispatchFailure(String),
}

#[derive(Default)]
struct MyPacketRouter {
    _source_node_id: NodeId,
}

impl MyPacketRouter {
    fn new(node_id: u32) -> Self {
        MyPacketRouter {
            _source_node_id: node_id.into(),
        }
    }
}

impl PacketRouter<(), DeviceUpdateError> for MyPacketRouter {
    fn handle_packet_from_radio(
        &mut self,
        _packet: FromRadio,
    ) -> std::result::Result<(), DeviceUpdateError> {
        debug!("handle_packet_from_radio called but not sure what to do");
        Ok(())
    }

    fn handle_mesh_packet(
        &mut self,
        _packet: MeshPacket,
    ) -> std::result::Result<(), DeviceUpdateError> {
        debug!("handle_mesh_packet called but not sure what to do here");
        Ok(())
    }

    fn source_node_id(&self) -> NodeId {
        self._source_node_id
    }
}

pub(crate) async fn meshtastic_loop(
    connection: Connection,
    tx: tokio::sync::mpsc::Sender<IPCMessage>,
    mut rx: tokio::sync::mpsc::Receiver<IPCMessage>,
) -> Result<()> {
    let stream_api = StreamApi::new();
    let mut decoded_listener;
    let connected_stream_api;
    match connection {
        Connection::TCP(ip, port) => {
            let tcp_stream = match utils::stream::build_tcp_stream(format!("{ip}:{port}")).await {
                Ok(sh) => sh,
                Err(e) => {
                    bail!(e);
                }
            };
            (decoded_listener, connected_stream_api) = stream_api.connect(tcp_stream).await;
        }
        Connection::Serial(device) => {
            let serial_stream = utils::stream::build_serial_stream(device, None, None, None)
                .expect("Unable to open serial port.");
            (decoded_listener, connected_stream_api) = stream_api.connect(serial_stream).await;
        }
        Connection::None => {
            panic!("Neither tcp nor serial selected for connection.");
        }
    }
    let config_id = utils::generate_rand_id();
    let mut _stream_api = connected_stream_api.configure(config_id).await?;
    info!("Connected to meshtastic node!");
    let mut packet_router = MyPacketRouter::new(0);
    loop {
        match decoded_listener.try_recv() {
            Ok(fr) => {
                if let Err(e) = tx.send(IPCMessage::FromRadio(fr)).await {
                    error!("Couldn't send FromRadio packet to mpsc: {e}");
                }
            }
            Err(_) => {}
        }
        match rx.try_recv() {
            Ok(inbound) => {
                match inbound {
                    IPCMessage::SendMessage(message) => {
                        if let Err(e) = _stream_api
                            .send_text(
                                &mut packet_router,
                                message.message,
                                message.destination,
                                true,
                                message.channel,
                            )
                            .await
                        {
                            error!("We tried to send a message but... nope: {e}");
                        }
                    }
                    IPCMessage::ToRadio(tr) => {
                        if let Err(e) = _stream_api.send_to_radio_packet(tr.payload_variant).await {
                            error!("We tried to send a ToRadio message directly but errored: {e}");
                        }
                    }
                    _ => {
                        warn!("Unknown ipc message sent into comms thread.");
                    }
                }
            }
                Err(_) => {}
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
        }
        Ok(())
    }
