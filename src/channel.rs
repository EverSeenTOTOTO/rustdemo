pub fn channel_demo() {
    let (tx, rx) = flume::unbounded();
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
