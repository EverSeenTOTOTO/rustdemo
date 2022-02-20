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
    main_tx: crossbeam_channel::Sender<String>,
}


impl ActorCounter {
    pub fn new(address: &str, rx: crossbeam_channel::Receiver<String>, tx: crossbeam_channel::Sender<String>, dispatcher: &str, reducer: &str, main_tx: crossbeam_channel::Sender<String>) -> ActorCounter {
        ActorCounter {
            address: address.to_string(),
            mail: SimpleChannelMail::new(address, rx, tx),
            dispatcher: dispatcher.to_string(),
            reducer: reducer.to_string(),
            main_tx
        }
    }

    // 计数器，收到count指令后计算质数，每算出一个向reducer发送一个
    pub fn run(&mut self) {
        println!("start {}, dispatcher is: {}, reducer is: {}", self.address, self.dispatcher, self.reducer);
        self.ready();
        loop {
            // std::thread::sleep(std::time::Duration::from_millis(300));
            if let Some(packet) = self.mail.recv() {
                match packet.command.as_str() {
                    "count range" => {
                        let range = packet.data.split(",").collect::<Vec<&str>>();
                        self.count(range[0].parse::<u64>().unwrap(), range[1].parse::<u64>().unwrap());
                    },
                    "done" => {
                        self.main_tx.send(self.address.clone()).unwrap();
                        break;
                    },
                    _ => {
                        println!("{} unknown command: {}", self.address, & packet.command);
                    }
                }
            }
        };
    }

    fn count(&self, from: u64, to: u64) {
        println!("{} count {},{}", self.address, from, to);

        for i in from..to {
            if is_prime(i) {
                self.mail.send("reducer", "prime value", &format!("{}", i));
            }
        }

        // 单次任务已完成
        self.ready();
    }

    fn ready(&self) {
        println!("{} ready", self.address);
        self.mail.send(&self.dispatcher, "worker ready", &self.address);
    }
}

#[derive(Debug)]
pub struct ActorReducer {
    pub address: String,
    pub required_prime_count: u64,
    pub received_primes: Vec<String>,
    dispatcher: String,
    mail: SimpleChannelMail,
    main_tx: crossbeam_channel::Sender<String>,
}

impl ActorReducer {
    pub fn new(address: &str, rx: crossbeam_channel::Receiver<String>, tx: crossbeam_channel::Sender<String>, required_prime_count: u64, dispatcher: &str, main_tx: crossbeam_channel::Sender<String>) -> ActorReducer {
        ActorReducer {
            address: address.to_string(),
            required_prime_count,
            received_primes: Vec::new(),
            mail: SimpleChannelMail::new(address, rx, tx),
            dispatcher: dispatcher.to_string(),
            main_tx
        }
    }

    // 收集器，收集质数到一定数目之后提交给dispatcher
    pub fn run(&mut self) {
        println!("start {}", self.address);
        loop {
            // std::thread::sleep(std::time::Duration::from_millis(300));
            if let Some(packet) = self.mail.recv() {
                match packet.command.as_str() {
                    "prime value" => {
                        println!("{} got {}", self.address, packet.data);
                        self.received_primes.push(packet.data);
                    },
                    _ => {
                        println!("{} unknown command: {}", self.address, & packet.command);
                    }
                }
            }

            if self.received_primes.len() >= self.required_prime_count as usize {
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

                self.main_tx.send(self.address.clone()).unwrap();
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
    main_tx: crossbeam_channel::Sender<String>,
    all_workers: Vec<String>, // 仅用于结束时停止worker
    mail: SimpleChannelMail,
}

impl ActorDispatcher {
    pub fn new(address: &str, rx: crossbeam_channel::Receiver<String>, tx: crossbeam_channel::Sender<String>, range_start: u64, offset: u64, main_tx: crossbeam_channel::Sender<String>) -> ActorDispatcher {
        ActorDispatcher {
            range_start,
            offset,
            address: address.to_string(),
            mail: SimpleChannelMail::new(address, rx, tx),
            all_workers: Vec::new(),
            main_tx
        }
    }

    pub fn save_worker_address(&mut self, address: &str) {
        self.all_workers.push(address.to_string());
    }

    // 分配器，在收到worker就绪的请求时分派任务，收到reducer完成的请求时终止
    pub fn run(&mut self) {
        println!("start {}", self.address);
        loop {
            // std::thread::sleep(std::time::Duration::from_millis(300));
            if let Some(packet) = self.mail.recv() {
                match packet.command.as_str() {
                    "worker ready" => {
                        println!("{} got {}", self.address, packet.data);
                        self.notify(&packet.data);
                    },
                    "done" => {
                        println!("final result: {}", packet.data);

                        for w in self.all_workers.clone() {
                            self.mail.send(&w, "done", "");
                        }

                        self.main_tx.send(self.address.clone()).unwrap();
                        break;
                    },
                    _ => {
                        println!("{} unknown command: {}", self.address, & packet.command);
                    }
                }
            }
        }
    }

    fn notify(&mut self, worker: &str) {
        self.mail.send(&worker, "count range", &format!("{},{}", self.range_start, self.range_start + self.offset));
        self.range_start += self.offset;
    }
}
