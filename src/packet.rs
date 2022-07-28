use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
pub struct Packet {
    pub from: String,
    pub to: String,
    pub command: String,
    pub data: String,
}

impl Packet {
    fn new(from: &str, to: &str, command: &str, data: &str) -> Packet {
        Packet {
            from: from.to_string(),
            to: to.to_string(),
            command: command.to_string(),
            data: data.to_string(),
        }
    }

    pub fn stringify(from: &str, to: &str, command: &str, data: &str) -> String {
        let res = json!({
            "from": from,
            "to": to,
            "command": command,
            "data": data,
        });

        res.to_string()
    }

    pub fn parse(msg: &str) -> Packet {
        let p: Packet = serde_json::from_str(msg).unwrap();
        p
    }
}
