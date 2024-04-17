use meshtastic::protobufs::from_radio::PayloadVariant;
use crate::ipc::IPCMessage;
use meshtastic::{
    api::StreamApi,
    utils
};
use anyhow::{
    Result,
    bail
};
use meshtastic::protobufs::FromRadio;
use tokio::sync::mpsc::error::TryRecvError;

pub(crate) async fn meshtastic_loop(tx: tokio::sync::mpsc::Sender<IPCMessage>) -> Result<()> {
    let stream_api = StreamApi::new();

    let tcp_stream = match utils::stream::build_tcp_stream("10.174.2.41:4403".to_string()).await {
        Ok(sh) => sh,
        Err(e) => {
            error!("Unable to connect to meshtastic host: {e}");
            bail!(e);
        }
    };
    let (mut decoded_listener, stream_api) = stream_api.connect(tcp_stream).await;


    let config_id = utils::generate_rand_id();
    let mut stream_api = stream_api.configure(config_id).await?;
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