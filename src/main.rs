// mod test_proxy;
// mod channel;
mod packet;
mod actor;

fn main() {
    let (tx, rx) = crossbeam_channel::unbounded();
    let (main_tx, main_rx) = crossbeam_channel::unbounded();

    let mut dispatcher = actor::ActorDispatcher::new("dispatcher", rx.clone(), tx.clone(), 1, 10000, main_tx.clone());
    let mut reducer = actor::ActorReducer::new("reducer", rx.clone(), tx.clone(), 10000, &dispatcher.address, main_tx.clone());

    let mut worker_a = actor::ActorCounter::new("worker A", rx.clone(), tx.clone(), &dispatcher.address, &reducer.address, main_tx.clone());
    let mut worker_b = actor::ActorCounter::new("worker B", rx.clone(), tx.clone(), &dispatcher.address, &reducer.address, main_tx.clone());

    dispatcher.save_worker_address(&worker_a.address);
    dispatcher.save_worker_address(&worker_b.address);

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
