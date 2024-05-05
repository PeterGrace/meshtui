# meshtui
A console-based Text-User-Interface (TUI) for Meshtastic.

`meshtui` allows you to connect either via serial port, or by ip address, to your meshtastic hardware and visualizes the information received.

## Installation and execution
  - Head over to the [Releases](https://github.com/PeterGrace/meshtui/releases) area and download the latest version for your platform.
  - execute `meshtui` with either the `-i <meshtastic-device-ip-address>` option for connecting over the network, or `-s [COMx|/dev/ttyXX]` to connect serially. 
  - Some people like seeing MQTT nodes alongside their RF nodes.  If you'd like to see mqtt, use the `--show-mqtt` command line argument.


## Functionality matrix
  - Messages
    - [X] can display messages
    - [X] can send message to channel 0   
    - [ ] can send messages to any channel
  - Channels
    - [X] can see a list of configured channels
    - [ ] can edit an existing channel
    - [ ] can import a channel via meshtastic-formed url
    - [ ] can produce a QR code scannable with phone to export channel info
  - Nodes
    - [X] can visualize the Node list
    - [X] can show traceroute data to node
    - [X] can show neigbhborinfo packet data for node
    - [X] can visualize via graph the relevant timeseries telemtry from mesh
    - [ ] can mute/ignore a node
  - Config
    - [X] Can visualize Device/Module config
    - [ ] Can update Device/Module config


## Navigating the application

| Esc/q | exits app | everywhere else |
| Tab | moves forward a tab | everywhere else |
| Shift-Tab | moves backwards a tab | everywhere else

The app starts out in the Messages tab.  You can navigate between tabs by using the Tab key to advance and Shift-Tab to move back a tab.

## Messages
![messages](messages.png?foo=bar)

| key | does |
| --- | ---- |
| esc/q | exits app |
| up/k | moves up one message |
| down/j | moves down one message |
| pgup | moves up one page or to the newest message |
| pgdn | moves down one page or to the last message |
| enter | toggles message send dialog |

The messages screen shows you a message list from your local mesh, and any stored messages that are in the Store And Forward buffer on the device.  The columns show the time the message was received, the source that sent the message, the channel name and number where it was received, and finally the message.  Messages are always sorted "newest at the top."

> What are those `seq XXX` messages I see on my mesh?

Those are rangefinder packets.  One of your mesh neighbors is using the Rangefinder module.  We may add a toggle to hide those in the future, but for now the app shows them.

If you'd like to send a message, hit the Enter key and a dialog will pop up.


### Send Message Dialog
![send-message](send-message.png?foo=bar)
| key | does |
| --- | ---- |
| Esc/q | closes send dialog |
| Enter | sends message |

In the send message dialog, you can type in a message to send to the mesh.  When you're ready to send, hit Enter and the message will send.  If you hit enter without writing a message, the window will close without sending anything.


## Nodes
![nodes](nodes.png?foo=bar)
| key | does |
| --- | ---- |
| esc/q | exits app |
| up/k | moves up one node |
| down/j | moves down one node |
| pgup | moves up one page or to the most-recently-heard node |
| pgdn | moves down one page or to the last node |
| enter | toggles node detail |

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


### Node Detail
![node-detail](node-detail.png?foo=bar)

| key | does | which screen |
| --- | ---- | ------------ |
| Esc/q | closes node detail | In node details screen |
| Tab | moves forward in graph list| in node details screen |
| Shift-Tab | moves backwards in graph list | in node details screen |

The node detail screen shows you data relevant to the node you've selected.  Beyond the basics that are reported in the node list, if the node publishes its neighbor list, you will see that in the upper right box of the screen.  If you hit F2 on this screen, a traceroute will be sent and the traceroute box on the lower right will populate with the response.  In the lower left box there are text representations of the data reported from the node, if they publish telemetry to the mesh.

## Channels
![channels](channels.png?foo=bar)

The channels tab shows the current channel config.  As of the time of this writing, the channels list functionality is 'view-only'.

## DeviceConfig
![device-config](device-config.png?foo=bar)
| key | does |
| left/h | moves backwards a sub-tab |
| right/l | moves forwards a sub-tab |

The DeviceConfig tab shows the configuration values from the device.  When meshtui starts, the current config is sent from the device to meshtui and we record the info.  Currently configuration changing is not implemented but may be in the future.

## ModulesConfig
![modules-config](modules-config.png?foo=bar)
| key | does |
| left/h | moves backwards a sub-tab |
| right/l | moves forwards a sub-tab |

Like the DeviceConfig tab, the ModulesConfig tab shows the configuration of all the sub-modules such as "Store and Forward", "Telemetry", or "NeighborInfo."  Editing is not yet supported, but may be in the future.

## About
![about](about.png?foo=bar)

Currently nothing lives here except the URL to get the meshtui repo.  I will probably add version info here at some point.
