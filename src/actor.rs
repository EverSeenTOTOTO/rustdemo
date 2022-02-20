use crate::packet::*;

fn is_prime(n: u64) -> bool {
    if n == 2 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }
    let mut i = 3;
    while i * i <= n {
        if n % i == 0 {
            return false;
        }
        i += 2;
    }
    return true;
}

// 模拟收发包，tx、rx是同一channel的两端，类似事件总线
#[derive(Debug)]
pub struct SimpleChannelMail {
    rx: crossbeam_channel::Receiver<String>,
    tx: crossbeam_channel::Sender<String>,
    pub address: String,
}

impl SimpleChannelMail {
    pub fn new(address: &str, rx: crossbeam_channel::Receiver<String>, tx: crossbeam_channel::Sender<String>) -> SimpleChannelMail {
        SimpleChannelMail {
            rx,
            tx,
            address: address.to_string(),
        }
    }

    pub fn send(&self, to: &str, command: &str, data: &str) {
        let msg = Packet::stringify(&self.address, to, command, data);

        println!("send {}", msg);

        self.tx.send(msg).unwrap();
    }

    pub fn recv(&self) -> Option<Packet> {
        if let Ok(msg) = self.rx.try_recv() {
            let packet = Packet::parse(&msg);

            // 取出事件，看是不是给自己的包，若不是，还需要放回总线
            if packet.to == self.address {
                // println!("{} recv {}", self.address, msg);
                return Some(packet)
            } else {
                self.put_back(&msg);
            }
        } else {
            println!("{} idle", self.address);
        }

        None
    }

    fn put_back(&self, msg: &str) {
        self.tx.send(msg.to_string()).unwrap();
    }
}

#[derive(Debug)]
pub struct ActorCounter {
    pub address: String,
    dispatcher: String,
    reducer: String,
    mail: SimpleChannelMail,
}

impl ActorCounter {
    pub fn new(address: &str, rx: crossbeam_channel::Receiver<String>, tx: crossbeam_channel::Sender<String>, dispatcher: &str, reducer: &str) -> ActorCounter {
        ActorCounter {
            address: address.to_string(),
            mail: SimpleChannelMail::new(address, rx, tx),
            dispatcher: dispatcher.to_string(),
            reducer: reducer.to_string(),
        }
    }

    pub fn run(&mut self) {
        println!("start {}, dispatcher is: {}, reducer is: {}", self.address, self.dispatcher, self.reducer);
        self.ready();
        loop {
            std::thread::sleep(std::time::Duration::from_millis(300));
            if let Some(packet) = self.mail.recv() {
                match packet.command.as_str() {
                    "count" => {
                        let range = packet.data.split(",").collect::<Vec<&str>>();
                        self.count(range[0].parse::<u64>().unwrap(), range[1].parse::<u64>().unwrap());
                    },
                    "done" => {
                        println!("{} stopped", self.address);
                        break;
                    },
                    _ => {}
                }
            }
        };
    }

    fn count(&self, from: u64, to: u64) {
        println!("{} count {},{}", self.address, from, to);

        for i in from..to {
            if is_prime(i) {
                self.mail.send("reducer", "value", &format!("{}", i));
            }
        }

        self.ready();
    }

    fn ready(&self) {
        println!("{} ready", self.address);
        self.mail.send(&self.dispatcher, "worker", &self.address);
    }
}

#[derive(Debug)]
pub struct ActorReducer {
    pub address: String,
    pub required_prime_count: u64,
    pub received_primes: Vec<String>,
    dispatcher: String,
    mail: SimpleChannelMail,
}

impl ActorReducer {
    pub fn new(address: &str, rx: crossbeam_channel::Receiver<String>, tx: crossbeam_channel::Sender<String>, required_prime_count: u64, dispatcher: &str) -> ActorReducer {
        ActorReducer {
            address: address.to_string(),
            required_prime_count,
            received_primes: Vec::new(),
            mail: SimpleChannelMail::new(address, rx, tx),
            dispatcher: dispatcher.to_string(),
        }
    }

    pub fn run(&mut self) {
        println!("start {}", self.address);
        loop {
            // std::thread::sleep(std::time::Duration::from_millis(300));
            if let Some(packet) = self.mail.recv() {
                match packet.command.as_str() {
                    "value" => {
                        println!("{} got {}", self.address, packet.data);
                        self.received_primes.push(packet.data);
                    },
                    _ => {}
                }
            }

            if self.received_primes.len() >= self.required_prime_count as usize {
                println!("{} done", self.address);

                // TODO: sort
                let result = self.received_primes
                    .clone()
                    .into_iter()
                    .reduce(|p, c| {
                        return format!("{},{}", p, c);
                    });
                // 发送结束信号
                match result {
                    Some(r) => {
                        self.mail.send(&self.dispatcher, "done", &r);
                    },
                    None => {
                        self.mail.send(&self.dispatcher, "done", "none");
                    }
                }
                break;
            }
        }
    }
}

#[derive(Debug)]
pub struct ActorDispatcher {
    pub address: String,
    pub range_start: u64,
    pub offset: u64,
    mail: SimpleChannelMail,
}

impl ActorDispatcher {
    pub fn new(address: &str, rx: crossbeam_channel::Receiver<String>, tx: crossbeam_channel::Sender<String>, range_start: u64, offset: u64) -> ActorDispatcher {
        ActorDispatcher {
            range_start,
            offset,
            address: address.to_string(),
            mail: SimpleChannelMail::new(address, rx, tx),
        }
    }

    pub fn run(&mut self) {
        println!("start {}", self.address);
        loop {
            std::thread::sleep(std::time::Duration::from_millis(300));
            if let Some(packet) = self.mail.recv() {
                match packet.command.as_str() {
                    "worker" => {
                        println!("{} got {}", self.address, packet.data);
                        self.notify(&packet.data);
                    },
                    "done" => {
                        println!("final result: {}", packet.data);
                        println!("{} stopped", self.address);
                        break;
                    },
                    _ => {}
                }
            }
        }
    }

    fn notify(&mut self, worker: &str) {
        self.mail.send(&worker, "count", &format!("{},{}", self.range_start, self.range_start + self.offset));
        self.range_start += self.offset;
    }
}
