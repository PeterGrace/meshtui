
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;

pub const DATE_FORMAT: &[BorrowedFormatItem<'_>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

pub const TICK_RATE: f64 = 10.0_f64;
pub const FRAME_RATE: f64 = 2.0_f64;

pub const MPSC_BUFFER_SIZE: usize = 100_usize;
pub const GPS_PRECISION_FACTOR: f32 = 0.0000001_f32;
pub const MAX_MSG_RETENTION: usize = 128_usize;


pub const NODE_HELP_TEXT: &str = r######"
The node screen shows a list of nodes as reported by your device.  The list is constantly sorted by
the most recent update to the node information that we've received.

Fields:
ID -- This is the 'address' of the node.  The value is a prefix and an 8-character hexadecimal
      number.  That number coincides to the "MAC Address" of the LORA transmitter of that node.
      Prefixes:
          ! - this is the normal prefix for a node id in Meshtastic.  You will see these on nodes
              where we have heard the complete node information from the node itself.

          * - this is informational from meshtui and isn't part of the Meshtastic specification.
              the asterisk shows that we have heard a reference to that node existing, but we haven't
              received actual nodeinfo from the node yet to fully populate its field in our database.

          ^ - this is a special indicator to show you your own node's record.

              Short -- Shortname of node.
               Long -- Long description for node.
         RF Details -- The SNR and RSSI of the received packets.
               Hops -- the reported number of hops away this device is in the mesh. "MQTT" if
                       the node is reachable only via MQTT.
          Neighbors -- If the node is reporting its neighbor table, this is the number of
                       reported neighbors it has.
           Distance -- If your local node has a GPS fix, this will show the distance to the
                       remote node from you.
        Lat/Lon/Alt -- Positional data for the remote node.
            Voltage -- Reported voltage of the remote node's power source
            Battery -- Battery percent left, or "Powered" if it is plugged in.
Last Heard NodeInfo -- The last time we received a full NodeInfo packet for this node.
        Last Update -- the last time we updated our database with any information about
                       this node.
"######;
