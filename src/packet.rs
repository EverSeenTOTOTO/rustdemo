#[derive(Debug)]
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
        let mut stringified = json::JsonValue::new_object();

        stringified["from"] = json::JsonValue::String(from.to_string());
        stringified["to"] = json::JsonValue::String(to.to_string());
        stringified["command"] = json::JsonValue::String(command.to_string());
        stringified["data"] = json::JsonValue::String(data.to_string());

        stringified.dump()
    }

    pub fn parse(msg: &str) -> Packet {
        let parsed: json::JsonValue = json::parse(msg).unwrap();
        let from = parsed["from"].as_str().unwrap();
        let to = parsed["to"].as_str().unwrap();
        let command = parsed["command"].as_str().unwrap();
        let data = parsed["data"].as_str().unwrap();

        Packet::new(from, to, command, data)
    }
}

