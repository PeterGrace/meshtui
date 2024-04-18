use std::collections::HashMap;
use meshtastic::Message;
use meshtastic::protobufs::{from_radio, mesh_packet, NeighborInfo, PortNum, Position, User};
use meshtastic::protobufs::telemetry::Variant;
use crate::ipc::IPCMessage;
use crate::tabs::nodes::ComprehensiveNode;
use crate::util;

pub async fn process_packet(packet: IPCMessage, node_list: HashMap<u32, ComprehensiveNode>) -> Option<(u32, ComprehensiveNode)> {
    if let IPCMessage::FromRadio(fr) = packet {
        if let Some(some_fr) = fr.payload_variant {
            match some_fr {
                from_radio::PayloadVariant::Packet(pa) => {
                    if let Some(payload) = pa.payload_variant {
                        match payload {
                            mesh_packet::PayloadVariant::Decoded(de) => {
                                match de.portnum() {
                                    PortNum::PositionApp => {
                                        let data = Position::decode(de.payload.as_slice()).unwrap();
                                        let mut cn = match node_list.contains_key(&pa.from) {
                                            true => node_list.get(&pa.from).unwrap().to_owned(),
                                            false => ComprehensiveNode::default()
                                        };
                                        info!("Updating Position for {} ({})",cn.clone().node_info.user.unwrap_or_else(|| User::default()).id,pa.from);
                                        cn.node_info.position = Some(data);
                                        cn.last_seen = util::get_secs();
                                        return Some((cn.node_info.num, cn));
                                    }
                                    PortNum::TelemetryApp => {
                                        let data = meshtastic::protobufs::Telemetry::decode(de.payload.as_slice()).unwrap();
                                        if let Some(v) = data.variant {
                                            match v {
                                                Variant::DeviceMetrics(dm) => {
                                                    let mut cn = match node_list.contains_key(&pa.from) {
                                                        true => node_list.get(&pa.from).unwrap().to_owned(),
                                                       false => {
                                                           warn!("We received DeviceMetrics from a node we don't have info on.  Ignoring.");
                                                           return None;
                                                       }
                                                    };
                                                    info!("Updating DeviceMetrics for {} ({})",cn.clone().node_info.user.unwrap_or_else(|| User::default()).id,pa.from);
                                                    cn.node_info.device_metrics = Some(dm);
                                                    cn.last_seen = util::get_secs();
                                                    return Some((cn.node_info.num, cn));
                                                }
                                                _ => { return None; }
                                                // Variant::EnvironmentMetrics(_) => {}
                                                // Variant::AirQualityMetrics(_) => {}
                                                // Variant::PowerMetrics(_) => {}
                                            }
                                        }
                                        return None;
                                    }
                                    PortNum::NeighborinfoApp => {
                                        let data = NeighborInfo::decode(de.payload.as_slice()).unwrap();
                                        let empty = ComprehensiveNode::default();
                                        for neighbor in data.neighbors.iter() {
                                            let d_cn = node_list.get(&data.node_id).map_or(empty.clone(), |v| v.clone());
                                            let n_cn = node_list.get(&neighbor.node_id).map_or(empty.clone(), |v| v.clone());

                                            let mut hub = "Unknown".to_string();
                                            let mut spoke = "Unknown".to_string();
                                            if let Some(d_user) = d_cn.node_info.user {
                                                hub = d_user.id;
                                            }
                                            if let Some(n_user) = n_cn.node_info.user {
                                                spoke = n_user.id;
                                            }
                                            info!("NeighborInfo: {hub} has neighbor {spoke}");
                                        }
                                        let mut cn = match node_list.get(&data.node_id) {
                                            None => {
                                                warn!("We received neighbor list from a node we don't have info on.  Ignoring.");
                                                return None;
                                            }
                                            Some(n) => {
                                                n.clone()
                                            }
                                        };
                                        cn.neighbors = data.neighbors;
                                        cn.last_seen = util::get_secs();
                                        return Some((cn.node_info.num, cn));
                                    }
                                    _ => { return None; }
                                    // PortNum::TracerouteApp => {}
                                    // PortNum::TextMessageApp => {}
                                    // PortNum::NodeinfoApp => {}
                                    // PortNum::RoutingApp => {}
                                    // PortNum::AdminApp => {}
                                    // PortNum::WaypointApp => {}
                                    // PortNum::ReplyApp => {}
                                    // PortNum::PaxcounterApp => {}
                                    // PortNum::StoreForwardApp => {}
                                    // PortNum::RangeTestApp => {}
                                }
                            }
                            mesh_packet::PayloadVariant::Encrypted(_) => {
                                info!("Received an encrypted packet.");
                                return None;
                            }
                        }
                    }
                    return None;
                }
                from_radio::PayloadVariant::MyInfo(mi) => {
                    info!("My node number is {:#?}", mi.my_node_num);
                    return None;
                }
                from_radio::PayloadVariant::NodeInfo(ni) => {
                    info!("Updating NodeInfo for {} ({})",ni.clone().user.unwrap_or_else(|| User::default()).id,ni.num);
                    let mut cn = ComprehensiveNode::default();
                    cn.node_info = ni.clone();
                    cn.last_seen = util::get_secs();
                    return Some((ni.num, cn));
                }
                _ => {
                    return None;
                }
                // from_radio::PayloadVariant::Config(_) => {}
                // from_radio::PayloadVariant::LogRecord(_) => {}
                // from_radio::PayloadVariant::ConfigCompleteId(_) => {}
                // from_radio::PayloadVariant::Rebooted(_) => {}
                // from_radio::PayloadVariant::ModuleConfig(_) => {}
                // from_radio::PayloadVariant::Channel(_) => {}
                // from_radio::PayloadVariant::QueueStatus(_) => {}
                // from_radio::PayloadVariant::XmodemPacket(_) => {}
                // from_radio::PayloadVariant::Metadata(_) => {}
                // from_radio::PayloadVariant::MqttClientProxyMessage(_) => {}
            }
        }
        return None;
    };
    return None;
}