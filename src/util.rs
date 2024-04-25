use std::time::{SystemTime, UNIX_EPOCH};
use crate::ipc::IPCMessage;
use anyhow::{bail, Result};

pub fn get_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub async fn send_to_radio(ipc: IPCMessage) -> Result<()> {
    let trm = crate::TO_RADIO_MPSC.write().await.clone().unwrap();
    if let Err(e) = trm
        .clone()
        .send(ipc)
        .await
    {
        bail!(e);
    }
    Ok(())
}
