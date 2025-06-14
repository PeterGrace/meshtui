use crate::app::DeviceConfiguration;
use crate::ipc::IPCMessage;
use crate::tabs::nodes::{ComprehensiveNode, TimeSeriesData};
use crate::util::get_secs;
use crate::{util, DEVICE_CONFIG};
use meshtastic::packet::PacketDestination;
use meshtastic::protobufs::config::PayloadVariant;
use meshtastic::protobufs::log_record::Level;
use meshtastic::protobufs::module_config::PayloadVariant as mpv;
use meshtastic::protobufs::{
    from_radio, mesh_packet, routing, telemetry, NeighborInfo, NodeInfo, PortNum, Position,
    RouteDiscovery, Routing, User,
};
use meshtastic::types::MeshChannel;
use meshtastic::Message;
use std::collections::HashMap;

pub(crate) enum PacketResponse {
    NodeUpdate(u32, Box<ComprehensiveNode>),
    UserUpdate(u32, User),
    InboundMessage(MessageEnvelope),
    OurAddress(u32),
}

#[derive(Debug, Clone)]
pub struct MessageEnvelope {
    pub(crate) timestamp: u32,
    pub(crate) source: Option<NodeInfo>,
    pub(crate) destination: PacketDestination,
    pub(crate) channel: MeshChannel,
    pub(crate) message: String,
    pub(crate) rx_rssi: i32,
    pub(crate) rx_snr: f32,
}

pub async fn process_packet(
    packet: IPCMessage,
    node_list: HashMap<u32, ComprehensiveNode>,
) -> Option<PacketResponse> {
    if let IPCMessage::FromRadio(fr) = packet {
        if let Some(some_fr) = fr.payload_variant {
            match some_fr {
                from_radio::PayloadVariant::Packet(pa) => {
                    if let Some(payload) = pa.clone().payload_variant {
                        match payload.clone() {
                            mesh_packet::PayloadVariant::Decoded(de) => {
                                match de.portnum() {
                                    PortNum::PositionApp => {
                                        let data = match Position::decode(de.payload.as_slice()) {
                                            Ok(d)=> d,
                                            Err(e) => {
                                                error!("Error decoding position: {}", e);
                                                return None;
                                            }
                                        };
                                        let mut cn = match node_list.contains_key(&pa.from) {
                                            true => node_list.get(&pa.from).unwrap().to_owned(),
                                            false => ComprehensiveNode::with_id(de.source),
                                        };
                                        info!(
                                            "Updating Position for {} ({})",
                                            cn.clone()
                                                .node_info
                                                .user
                                                .unwrap_or_else(User::default)
                                                .id,
                                            pa.from
                                        );
                                        cn.node_info.position = Some(data);
                                        cn.last_seen = util::get_secs();
                                        cn.last_rssi = pa.rx_rssi;
                                        cn.last_snr = pa.rx_snr;
                                        return Some(PacketResponse::NodeUpdate(
                                            cn.node_info.num,
                                            Box::new(cn),
                                        ));
                                    }
                                    PortNum::TelemetryApp => {
                                        let data = match meshtastic::protobufs::Telemetry::decode(de.payload.as_slice()) {
                                            Ok(d) => d,
                                            Err(e) => {
                                                error!("Error decoding telemetry: {e}");
                                                return None
                                            }
                                        };
                                        if let Some(v) = data.variant {
                                            match v {
                                                telemetry::Variant::EnvironmentMetrics(env) => {
                                                    let mut cn = match node_list
                                                        .contains_key(&pa.from)
                                                    {
                                                        true => node_list
                                                            .get(&pa.from)
                                                            .unwrap()
                                                            .to_owned(),
                                                        false => {
                                                            ComprehensiveNode::with_id(pa.from)
                                                        }
                                                    };
                                                    info!("Received EnvironmentalMetrics from !{:x} ({})", pa.from, pa.from);
                                                    cn.timeseries.push_back(
                                                        TimeSeriesData {
                                                            timestamp: get_secs(),
                                                            environment: env.clone(),
                                                            rssi: pa.rx_rssi as f64,
                                                            snr: pa.rx_snr as f64,
                                                            ..Default::default()
                                                        });
                                                    if cn.timeseries_start == 0 {
                                                        cn.timeseries_start = get_secs();
                                                    };
                                                    cn.last_seen = util::get_secs();
                                                    cn.last_rssi = pa.rx_rssi;
                                                    cn.last_snr = pa.rx_snr;
                                                    return Some(PacketResponse::NodeUpdate(
                                                        cn.node_info.num,
                                                        Box::new(cn),
                                                    ));
                                                    
                                                }
                                                telemetry::Variant::DeviceMetrics(dm) => {
                                                    let mut cn = match node_list
                                                        .contains_key(&pa.from)
                                                    {
                                                        true => node_list
                                                            .get(&pa.from)
                                                            .unwrap()
                                                            .to_owned(),
                                                        false => {
                                                            ComprehensiveNode::with_id(pa.from)
                                                        }
                                                    };
                                                    info!(
                                                        "Updating DeviceMetrics for {} ({})",
                                                        cn.clone()
                                                            .node_info
                                                            .user
                                                            .unwrap_or_else(User::default)
                                                            .id,
                                                        pa.from
                                                    );
                                                    cn.node_info.device_metrics = Some(dm.clone());
                                                    cn.timeseries.push_back(
                                                        TimeSeriesData {
                                                            timestamp: get_secs(),
                                                            device: dm.clone(),
                                                            rssi: pa.rx_rssi as f64,
                                                            snr: pa.rx_snr as f64,
                                                            ..Default::default()
                                                        });
                                                    if cn.timeseries_start == 0 {
                                                        cn.timeseries_start = get_secs();
                                                    };
                                                    cn.last_seen = util::get_secs();
                                                    cn.last_rssi = pa.rx_rssi;
                                                    cn.last_snr = pa.rx_snr;
                                                    return Some(PacketResponse::NodeUpdate(
                                                        cn.node_info.num,
                                                        Box::new(cn),
                                                    ));
                                                }
                                                _ => {
                                                    return None;
                                                } // Variant::EnvironmentMetrics(_) => {}
                                                  // Variant::AirQualityMetrics(_) => {}
                                                  // Variant::PowerMetrics(_) => {}
                                            }
                                        }
                                        return None;
                                    }
                                    PortNum::NeighborinfoApp => {
                                        let data =
                                            match NeighborInfo::decode(de.payload.as_slice()) {
                                                Ok(d) => d,
                                                Err(e) => {
                                                    error!("Error decoding neighbor info: {}", e);
                                                    return None;
                                                }
                                            };
                                        let empty = ComprehensiveNode::with_id(de.source);
                                        for neighbor in data.neighbors.iter() {
                                            let d_cn = node_list
                                                .get(&data.node_id)
                                                .map_or(empty.clone(), |v| v.clone());
                                            let n_cn = node_list
                                                .get(&neighbor.node_id)
                                                .map_or(empty.clone(), |v| v.clone());

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
                                                ComprehensiveNode::with_id(pa.from)
                                            }
                                            Some(n) => n.clone(),
                                        };
                                        cn.neighbors = data.neighbors;
                                        cn.last_seen = util::get_secs();
                                        cn.last_rssi = pa.rx_rssi;
                                        cn.last_snr = pa.rx_snr;
                                        return Some(PacketResponse::NodeUpdate(
                                            cn.node_info.num,
                                            Box::new(cn),
                                        ));
                                    }
                                    PortNum::NodeinfoApp => {
                                        let data = match User::decode(de.payload.as_slice())
                                        {
                                            Ok(d) => d,
                                            Err(e) => {
                                                error!("Error decoding user: {}", e);
                                                return None;
                                            }
                                        };
                                        info!(
                                            "Received node info update for {} ({})",
                                            data.id, pa.from
                                        );
                                        let nid = u32::from_str_radix(
                                            data.id.clone().trim_start_matches('!'),
                                            16,
                                        )
                                        .unwrap_or(0_u32);
                                        if nid == 0 {
                                            error!("Received a node update but the node string ({}) is not parseable hexadecimal",data.id.clone());
                                            return None;
                                        }

                                        return Some(PacketResponse::UserUpdate(nid, data));
                                    }
                                    PortNum::RoutingApp => {
                                        let data = match Routing::decode(de.payload.as_slice()) {
                                            Ok(d) => d,
                                            Err(e) => {
                                                error!("Error decoding routing: {}", e);
                                                return None;
                                            }
                                        };
                                        if let Some(v) = data.variant {
                                            match v {
                                                routing::Variant::RouteRequest(_r) => {
                                                    info!("RouteRequest");
                                                }
                                                routing::Variant::RouteReply(_rr) => {
                                                    info!("RouteReply")
                                                }
                                                routing::Variant::ErrorReason(er) => match er {
                                                    0 => {
                                                        let _from_id = pa.clone().from;
                                                        let _to_id = pa.clone().to;

                                                        debug!("Routing Message: Outbound message id {} successfully transmitted" ,de.request_id);
                                                    }
                                                    _ => {
                                                        info!("Routing Error: message trace id {} has errorcode {}", de.request_id, er);
                                                    }
                                                },
                                            }
                                        }
                                    }
                                    PortNum::TracerouteApp => {
                                        let val_resp =
                                            RouteDiscovery::decode(de.payload.as_slice());
                                        if let Ok(route) = val_resp {
                                            let from_id = pa.clone().from;
                                            let to_id = pa.clone().to;
                                            let mut cn = match node_list.get(&from_id) {
                                                None => {
                                                    error!("{:#?}", pa.clone());
                                                    return None;
                                                }
                                                Some(n) => n.clone(),
                                            };
                                            cn.route_list.insert(to_id, route.clone().route);
                                            info!(
                                                "updating route table to {:#?} for !{:x}->!{:x}",
                                                route.route, from_id, to_id
                                            );
                                            return Some(PacketResponse::NodeUpdate(cn.id, Box::new(cn)));
                                        }
                                    }
                                    PortNum::ReplyApp => {
                                        info!("We were just pinged.");
                                    }

                                    PortNum::TextMessageApp => {
                                        if let Ok(message) = String::from_utf8(de.payload) {
                                            let source_ni = match node_list.get(&pa.from) {
                                                Some(s) => s.clone().node_info,
                                                None => {
                                                    info!(
                                                        "Could not find node info for id {}",
                                                        pa.from
                                                    );
                                                    return None;
                                                }
                                            };
                                            let _dest_ni =
                                                node_list.get(&pa.to).map(|s| s.clone().node_info);
                                            let destinated: PacketDestination = match pa.to {
                                                0 => PacketDestination::Local,
                                                u32::MAX => PacketDestination::Broadcast,
                                                s => PacketDestination::Node(s.into()),
                                            };

                                            return Some(PacketResponse::InboundMessage(
                                                MessageEnvelope {
                                                    timestamp: pa.rx_time,
                                                    source: Some(source_ni),
                                                    destination: destinated,
                                                    channel: MeshChannel::from(pa.channel),
                                                    message,
                                                    rx_rssi: pa.rx_rssi,
                                                    rx_snr: pa.rx_snr,
                                                },
                                            ));
                                        } else {
                                            warn!(
                                                "Unable to decode text message to utf8 from ({})",
                                                de.source
                                            );
                                        }
                                    }
                                    _ => {
                                        error!("{:#?}", de);
                                        return None;
                                    } // PortNum::AdminApp => {}
                                      // PortNum::WaypointApp => {}

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
                    return Some(PacketResponse::OurAddress(mi.my_node_num));
                }
                from_radio::PayloadVariant::NodeInfo(ni) => {
                    info!(
                        "Updating NodeInfo for {} ({})",
                        ni.clone().user.unwrap_or_else(User::default).id,
                        ni.num
                    );
                    let mut cn = ComprehensiveNode::with_id(ni.num);
                    cn.node_info = ni.clone();
                    cn.last_seen = util::get_secs();
                    cn.last_rssi = 0;
                    cn.last_snr = ni.snr;

                    return Some(PacketResponse::NodeUpdate(ni.num, Box::new(cn)));
                }
                from_radio::PayloadVariant::Config(cfg) => {
                    info!("Receiving DeviceConfig from device.");
                    match cfg.payload_variant {
                        None => {}
                        Some(s) => {
                            let mut f = DEVICE_CONFIG.write().await;
                            if f.is_none() {
                                *f = Some(DeviceConfiguration::default());
                            }
                            let mut devcfg = f.clone().unwrap();
                            match s {
                                PayloadVariant::Device(d) => devcfg.device = d,
                                PayloadVariant::Position(p) => devcfg.position = p,
                                PayloadVariant::Power(p) => devcfg.power = p,
                                PayloadVariant::Network(n) => devcfg.network = n,
                                PayloadVariant::Display(d) => devcfg.display = d,
                                PayloadVariant::Lora(l) => devcfg.lora = l,
                                PayloadVariant::Bluetooth(b) => devcfg.bluetooth = b,
                            }
                            devcfg.last_update = get_secs();
                            *f = Some(devcfg);
                        }
                    }
                }
                from_radio::PayloadVariant::LogRecord(v) => {
                    match v.level() {
                        Level::Unset => {
                            info!("Log Message: {}", v.message)
                        }
                        Level::Critical => {
                            error!("Log Message: {}", v.message)
                        }
                        Level::Error => {
                            error!("Log Message: {}", v.message)
                        }
                        Level::Warning => {
                            warn!("Log Message: {}", v.message)
                        }
                        Level::Info => {
                            info!("Log Message: {}", v.message)
                        }
                        Level::Debug => {
                            debug!("Log Message: {}", v.message)
                        }
                        Level::Trace => {
                            trace!("Log Message: {}", v.message)
                        }
                    }
                    return None;
                }
                //from_radio::PayloadVariant::ConfigCompleteId(_) => {}
                from_radio::PayloadVariant::Rebooted(v) => {
                    if v {
                        info!("Device has reported a reboot");
                    }
                    return None;
                }
                from_radio::PayloadVariant::ModuleConfig(module_obj) => {
                    info!("Receiving ModulesConfig from device.");
                    if let Some(module) = module_obj.payload_variant {
                        let mut f = DEVICE_CONFIG.write().await;
                        if f.is_none() {
                            *f = Some(DeviceConfiguration::default());
                        }
                        let mut devcfg = f.clone().unwrap();

                        match module {
                            mpv::Mqtt(o) => devcfg.mqtt = o,
                            mpv::Serial(o) => devcfg.serial = o,
                            mpv::ExternalNotification(o) => devcfg.external_notification = o,
                            mpv::StoreForward(o) => devcfg.store_forward = o,
                            mpv::RangeTest(o) => devcfg.range_test = o,
                            mpv::Telemetry(o) => devcfg.telemetry = o,
                            mpv::CannedMessage(o) => devcfg.canned_message = o,
                            mpv::Audio(o) => devcfg.audio = o,
                            mpv::RemoteHardware(o) => devcfg.remote_hardware = o,
                            mpv::NeighborInfo(o) => devcfg.neighbor_info = o,
                            mpv::AmbientLighting(o) => devcfg.ambient_lighting = o,
                            mpv::DetectionSensor(o) => devcfg.detection_sensor = o,
                            mpv::Paxcounter(o) => devcfg.paxcounter = o,
                        }
                        devcfg.last_update = get_secs();
                        *f = Some(devcfg);
                    }
                }
                from_radio::PayloadVariant::ConfigCompleteId(u) => {
                    info!(
                        "We've received all config from the device! (Checksum {})",
                        u
                    );
                }
                from_radio::PayloadVariant::Channel(c) => {
                    let mut channelpacket = c.clone();
                    if let Some(mut channel) = channelpacket.settings.clone() {
                        let mut f = DEVICE_CONFIG.write().await;
                        if f.is_none() {
                            *f = Some(DeviceConfiguration::default());
                        }
                        let mut devcfg = f.clone().unwrap();
                        if c.index == 0 && channel.name.is_empty() && channel.psk == [1] {
                            channel.name = "LongFast (Default)".to_string();
                        };
                        channelpacket.settings = Some(channel.clone());
                        info!(
                            "Storing channel config for {} (Ch: {})",
                            channel.name, c.index
                        );
                        devcfg.channels.insert(c.index, channelpacket.clone());

                        devcfg.last_update = get_secs();
                        *f = Some(devcfg);
                    }
                }
                from_radio::PayloadVariant::QueueStatus(v) => {
                    debug!(
                        "QueueStatus: res {}/free {}/maxlen {}/mesh_packet_id {}",
                        v.res, v.free, v.maxlen, v.mesh_packet_id
                    );
                    return None;
                }
                from_radio::PayloadVariant::XmodemPacket(v) => {
                    info!("{:#?}", v);
                    return None;
                }
                from_radio::PayloadVariant::Metadata(v) => {
                    info!("Device firmware version: {}", v.firmware_version);
                    return None;
                }
                from_radio::PayloadVariant::MqttClientProxyMessage(v) => {
                    info!("{:#?}", v);
                    return None;
                }
            }
        }
        return None;
    };
    None
}
