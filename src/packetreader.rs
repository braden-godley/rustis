use regex::Regex;

const VERSION: &'static str = "1.0.0";

/* Packet format:

Rustis <VERSION>
<COMMAND>
<DATA1>
<DATA2>
<DATA3>
...

*/

// Represents a packet sent by the client
#[derive(Debug)]
pub enum RequestPacket {
    Publish {
        channel: String,
        message: String,
    },
    Subscribe {
        channel: String,
    },
    Set {
        key: String,
        value: String,
    },
    // Get {
    //     key: String,
    // },
    Unknown,
    Invalid {
        error: String,
    },
}


impl RequestPacket {
    pub fn new(buf: String) -> Self {
        let lines: Vec<_> = buf.split("\n").collect();

        let version = if let Some(line1) = lines.get(0) {
            let re = Regex::new(r"^Rustis (\d{1,3}\.\d{1,4}\.\d{1,4})$").unwrap();

            if let Some(caps) = re.captures(line1) {
                Some(caps.get(1).unwrap().as_str())
            } else {
                None
            }
        } else {
            None
        };

        if let Some(version) = version {
            if version != VERSION {
                return RequestPacket::Invalid {
                    error: String::from("version mismatch"),
                }
            }
        } else {
            return RequestPacket::Invalid {
                error: String::from("missing headers")
            };
        }

        let command = lines.get(1);

        if let Some(command) = command {
            match *command {
                "publish" => {
                    let channel = lines.get(2);
                    let message = lines.get(3..);
                    if let Some(channel) = channel {
                        if let Some(message) = message {
                            let message = message.join("\n");
                            RequestPacket::Publish{ channel: channel.to_string(), message: message.to_string() }
                        } else {
                            RequestPacket::Invalid {
                                error: String::from("missing message")
                            }
                        }
                    } else {
                        RequestPacket::Invalid {
                            error: String::from("missing channel")
                        }
                    }
                },
                "subscribe" => { 
                    let channel = lines.get(2);
                    if let Some(channel) = channel {
                        RequestPacket::Subscribe { channel: channel.to_string() } 
                    } else {
                        RequestPacket::Invalid {
                            error: String::from("missing channel")
                        }
                    }
                },
                "set" => {
                    let key = lines.get(2);
                    let value = lines.get(3..);
                    if let Some(key) = key {
                        if let Some(value) = value {
                            let value = value.join("\n");
                            RequestPacket::Set{ key: key.to_string(), value: value.to_string() }
                        } else {
                            RequestPacket::Invalid {
                                error: String::from("missing value")
                            }
                        }
                    } else {
                        RequestPacket::Invalid {
                            error: String::from("missing key")
                        }
                    }
                },
                _ => RequestPacket::Unknown,
            }
        } else {
            RequestPacket::Unknown
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn publish_packet() {
        let mut buf = String::new();
        buf.push_str("Rustis 1.0.0\n");
        buf.push_str("publish\n");
        buf.push_str("channel\n");
        buf.push_str("message");

        let packet = RequestPacket::new(buf);

        match packet {
            RequestPacket::Publish { .. } => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn invalid_version() {
        let mut buf = String::new();
        buf.push_str("Rustis 99.99.99\n");
        buf.push_str("publish\n");
        buf.push_str("channel\n");
        buf.push_str("message\n");

        let packet = RequestPacket::new(buf);

        match packet {
            RequestPacket::Invalid { .. } => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn unknown_command() {
        let mut buf = String::new();
        buf.push_str("Rustis 1.0.0\n");
        buf.push_str("heebee\n");

        let packet = RequestPacket::new(buf);

        match packet {
            RequestPacket::Unknown => assert!(true),
            _ => assert!(false),
        }
    }
}
