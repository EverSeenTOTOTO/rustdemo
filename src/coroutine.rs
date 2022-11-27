#![allow(non_snake_case)]
use std::{
    sync::mpsc::{sync_channel, SyncSender},
    thread,
    time::Duration,
};

fn setTimeout(callback: &dyn Fn(), timeout: u64) {
    std::thread::sleep(Duration::from_millis(timeout));
    callback();
}

type Lambda<Param> = Box<dyn Fn(Param) + Send + Sync>;

// Rust does not support recursive lambda function
struct Closure<'a> {
    lambda: &'a dyn Fn(&Closure),
}

// How to convert to SyncSender<Lambda<&Closure>> with appropriate lifetime?
type Yield = SyncSender<Box<dyn Fn(&Closure) + Send + Sync>>;

fn autoRun(gen: Lambda<Yield>) {
    let (tx, rx): (Yield, _) = sync_channel(1);

    let run = Closure {
        lambda: &|this| {
            if let Ok(result) = rx.recv() {
                result(this);
            }
        },
    };

    thread::spawn(move || gen(tx));
    (run.lambda)(&run);
}

pub fn test_yield() {
    autoRun(Box::new(|tx| {
        tx.send(Box::new(|next| {
            setTimeout(
                &|| {
                    println!("2");
                    (next.lambda)(next);
                },
                1000,
            );
        }))
        .unwrap();

        tx.send(Box::new(|next| {
            setTimeout(
                &|| {
                    println!("3");
                    (next.lambda)(next);
                },
                1000,
            );
        }))
        .unwrap();
    }));
}
