pub fn test_channel() {
    let (tx, rx) = crossbeam_channel::unbounded();
    let threads: Vec<_> = (0..2)
        .map(|id| {
            let tx = tx.clone();
            let rx = rx.clone();
            std::thread::spawn(move || {
                println!("thread start: {}", id);
                if id % 2 == 0 {
                    loop {
                        println!("send");
                        tx.send("recv").unwrap();
                        std::thread::sleep(std::time::Duration::from_secs(1));
                    }
                } else {
                    loop {
                        if let Ok(msg) = rx.recv() {
                            println!("{}", msg)
                        }
                    }
                }
            })
        })
        .collect();

    for thread in threads {
        thread.join().unwrap();
    }
}

pub struct Consumer {
    rx: crossbeam_channel::Receiver<i32>,
}

impl Consumer {
    pub fn new(rx: crossbeam_channel::Receiver<i32>) -> Self {
        Self { rx }
    }
}

impl Iterator for Consumer {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(val) = self.rx.try_recv() {
            println!("consumed {}", val);
            return Some(val);
        }
        None
    }
}

pub struct Producer {
    tx: crossbeam_channel::Sender<i32>,
    current: i32,
}

impl Producer {
    pub fn new(tx: crossbeam_channel::Sender<i32>) -> Self {
        Producer { tx, current: 0 }
    }
}

impl Iterator for Producer {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        println!("produced {}", self.current);
        self.tx.try_send(self.current).ok();
        self.current += 1;

        Some(self.current)
    }
}

pub fn test_iter() {
    let (tx, rx) = crossbeam_channel::bounded(1);
    let mut producer = Producer::new(tx);
    let mut consumer = Consumer::new(rx);

    producer.next();
    consumer.next();
    producer.next();
    producer.next();
}

pub fn infinite_produce() {
    let (tx, _) = crossbeam_channel::unbounded();
    let producer = Producer::new(tx);

    producer.for_each(drop);
}
