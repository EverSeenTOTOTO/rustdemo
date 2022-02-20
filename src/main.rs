// mod test_proxy;
// mod channel;
mod packet;
mod actor;

fn main() {
    let (tx, rx) = crossbeam_channel::unbounded();
 
    let mut dispatcher = actor::ActorDispatcher::new("dispatcher", rx.clone(), tx.clone(), 1, 1000);
    let mut reducer = actor::ActorReducer::new("reducer", rx.clone(), tx.clone(), 500, "dispatcher");

    let mut worker_a = actor::ActorCounter::new("worker A", rx.clone(), tx.clone(), "dispatcher", "reducer");
    let mut worker_b = actor::ActorCounter::new("worker B", rx.clone(), tx.clone(), "dispatcher", "reducer");

    std::thread::spawn(move || {
        worker_a.run();
    });
    std::thread::spawn(move || {
        worker_b.run();
    });
    
    std::thread::spawn(move || {
        dispatcher.run();
    });

    std::thread::spawn(move || {
        reducer.run();
    }).join().unwrap();
}
