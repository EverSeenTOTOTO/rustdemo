use crate::packet::*;

type Sender = crossbeam_channel::Sender<String>;
type Receiver = crossbeam_channel::Receiver<String>;

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
    true
}

// 模拟收发包，tx、rx是同一channel的两端，类似事件总线
#[derive(Debug)]
pub struct SimpleChannelMail {
    rx: Receiver,
    tx: Sender,
    pub address: String,
}

impl SimpleChannelMail {
    pub fn new(address: &str, rx: Receiver, tx: Sender) -> SimpleChannelMail {
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
            // 缺点是会改变事件的顺序
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
    dispatcher_address: Option<String>,
    reducer_address: Option<String>,
    mail: SimpleChannelMail,
    main_tx: Sender,
}


impl ActorCounter {
    pub fn new(address: &str, rx: Receiver, tx: Sender, main_tx: Sender) -> ActorCounter {
        ActorCounter {
            address: address.to_string(),
            mail: SimpleChannelMail::new(address, rx, tx),
            main_tx,
            dispatcher_address: None,
            reducer_address: None,
        }
    }

    pub fn save_dispatcher_address(&mut self, address: &str) {
        self.dispatcher_address = Some(address.to_string());
    }

    pub fn save_reducer_address(&mut self, address: &str) {
        self.reducer_address = Some(address.to_string());
    }

    pub fn get_dispatcher_address(&self) -> String {
        if let Some(address) = &self.dispatcher_address {
            address.to_string()
        } else {
            panic!("{}", format!("{}: dispatcher address is None", self.address));
        }
    }

    pub fn get_reducer_address(&self) -> String {
        if let Some(address) = &self.reducer_address {
            address.to_string()
        } else {
            panic!("{}", format!("{}: reducer address is None", self.address));
        }
    }

    // 计数器，收到count指令后计算质数，每算出一个向reducer发送一个
    pub fn run(&mut self) {
        let dispatcher = self.get_dispatcher_address();
        let reducer = self.get_reducer_address();

        println!("start {}, dispatcher is: {}, reducer is: {}", self.address, dispatcher, reducer);
        self.ready();
        loop {
            // std::thread::sleep(std::time::Duration::from_millis(300));
            if let Some(packet) = self.mail.recv() {
                match packet.command.as_str() {
                    "count range" => {
                        let range = packet.data.split(',').collect::<Vec<&str>>();
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
                std::thread::sleep(std::time::Duration::from_millis(300));
                self.mail.send(&self.get_reducer_address(), "prime value", &format!("{}", i));
            }
        }

        // 单次任务已完成
        self.ready();
    }

    fn ready(&self) {
        println!("{} ready", self.address);
        self.mail.send(&self.get_dispatcher_address(), "worker ready", &self.address);
    }
}

#[derive(Debug)]
pub struct ActorReducer {
    pub address: String,
    pub required_prime_count: u64,
    pub received_primes: Vec<String>,
    dispatcher_address: Option<String>,
    mail: SimpleChannelMail,
    main_tx: Sender,
}

impl ActorReducer {
    pub fn new(address: &str, rx: Receiver, tx: Sender, main_tx: Sender, required_prime_count: u64) -> ActorReducer {
        ActorReducer {
            address: address.to_string(),
            required_prime_count,
            received_primes: Vec::new(),
            mail: SimpleChannelMail::new(address, rx, tx),
            dispatcher_address: None,
            main_tx
        }
    }

    pub fn save_dispatcher_address(&mut self, address: &str) {
        self.dispatcher_address = Some(address.to_string());
    }

    pub fn get_dispatcher_address(&self) -> String {
        if let Some(address) = &self.dispatcher_address {
            address.to_string()
        } else {
            panic!("{}", format!("{}: dispatcher address is None", self.address));
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
                let mut result = self.received_primes
                    .clone()
                    .into_iter()
                    .collect::<Vec<String>>();

                result.sort_by(|a, b| a.parse::<u64>().unwrap().cmp(&b.parse::<u64>().unwrap()));

                let dispatcher = self.get_dispatcher_address();
                // 发送结束信号
                self.mail.send(&dispatcher, "done", &result.join(","));

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
    main_tx: Sender,
    all_workers: Vec<String>, // 仅用于结束时停止worker
    mail: SimpleChannelMail,
}

impl ActorDispatcher {
    pub fn new(address: &str, rx: Receiver, tx: Sender, main_tx: Sender, range_start: u64, offset: u64) -> ActorDispatcher {
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
        self.mail.send(worker, "count range", &format!("{},{}", self.range_start, self.range_start + self.offset));
        self.range_start += self.offset;
    }
}

pub fn test_actor_multi_thread() {
    let (tx, rx) = crossbeam_channel::unbounded();
    let (main_tx, main_rx) = crossbeam_channel::unbounded();

    let mut dispatcher = ActorDispatcher::new("dispatcher", rx.clone(), tx.clone(), main_tx.clone(), 0, 1000);
    let mut reducer = ActorReducer::new("reducer", rx.clone(), tx.clone(), main_tx.clone(), 10);

    let mut worker_a = ActorCounter::new("worker A", rx.clone(), tx.clone(), main_tx.clone());
    let mut worker_b = ActorCounter::new("worker B", rx.clone(), tx.clone(), main_tx.clone());

    // ugly
    reducer.save_dispatcher_address(&dispatcher.address);

    dispatcher.save_worker_address(&worker_a.address);
    dispatcher.save_worker_address(&worker_b.address);

    worker_a.save_dispatcher_address(&dispatcher.address);
    worker_a.save_reducer_address(&reducer.address);
    worker_b.save_dispatcher_address(&dispatcher.address);
    worker_b.save_reducer_address(&reducer.address);

    let threads = vec![
        std::thread::spawn(move || {
            let mut closed = Vec::new();
            loop {
                if let Ok(name) = main_rx.try_recv() {
                    println!("{} closed", name);
                    closed.push(name);

                    // if all threads are closed, drop remaining msg on bus and exit
                    if closed.len() == 4 {
                        println!("drop {} messages", rx.len());
                        drop(rx);
                        break;
                    }
                } else {
                    println!("{} remaining", rx.len());
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                }
            }
        }),
        std::thread::spawn(move || {
            worker_a.run();
        }),
        std::thread::spawn(move || {
            worker_b.run();
        }),
        std::thread::spawn(move || {
            dispatcher.run();
        }),
        std::thread::spawn(move || {
            reducer.run();
        }),
    ];

    for t in threads {
        t.join().unwrap();
    }
}
