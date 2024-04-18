use crate::ipc::IPCMessage;
use meshtastic::{
    api::StreamApi,
    utils
};
use anyhow::{
    Result,
    bail
};
use meshtastic::api::ConnectedStreamApi;
use meshtastic::api::state::Connected;
use meshtastic::packet::PacketReceiver;
use crate::app::Connection;


pub(crate) async fn meshtastic_loop(connection: Connection, tx: tokio::sync::mpsc::Sender<IPCMessage>) -> Result<()> {

    let stream_api = StreamApi::new();
    let mut decoded_listener;
    let mut connected_stream_api;
    match connection {
        Connection::TCP(ip, port) => {
            let tcp_stream = match utils::stream::build_tcp_stream("10.174.2.41:4403".to_string()).await {
                Ok(sh) => sh,
                Err(e) => {
                    error!("Unable to connect to meshtastic host: {e}");
                    bail!(e);
                }
            };
            (decoded_listener, connected_stream_api) = stream_api.connect(tcp_stream).await;
        }
        Connection::Serial(device) => {
            let serial_stream = utils::stream::build_serial_stream(device, None, None, None).expect("Unable to open serial port.");
            (decoded_listener, connected_stream_api) = stream_api.connect(serial_stream).await;
        },
        Connection::None  => {
            panic!("Neither tcp nor serial selected for connection.");
        }
    }




    let config_id = utils::generate_rand_id();
    let _stream_api = connected_stream_api.configure(config_id).await?;
    //stream_api.update_user()
    info!("Connected to meshtastic node!");

    loop {

        match decoded_listener.try_recv() {
            Ok(fr) => {
                if let Err(e) = tx.send(IPCMessage::FromRadio(fr)).await {
                    error!("Couldn't send FromRadio packet to mpsc: {e}");
                }
            },
            Err(_) => {}
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
    }
    Ok(())
}