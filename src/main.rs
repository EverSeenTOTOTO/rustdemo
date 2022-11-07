#![allow(non_snake_case)]
use std::{
    sync::{
        mpsc::{sync_channel, SyncSender},
        Arc,
    },
    thread,
    time::Duration,
};

fn setTimeout<'s>(callback: &'s dyn Fn(), timeout: u64) {
    std::thread::sleep(Duration::from_millis(timeout));
    callback();
}

type Lambda<'a, Param> = Arc<dyn Fn(Param) + Send + Sync + 'a>;

// Rust does not support recursive lambda function
struct Closure<'a> {
    lambda: &'a dyn Fn(&Closure),
}

// How to convert to SyncSender<Lambda<&Closure>> with appropriate lifetime?
type Yield = SyncSender<Arc<dyn Fn(&Closure) + Send + Sync>>;

fn autoRun(gen: Lambda<'static, Yield>) {
    let (tx, rx): (Yield, _) = sync_channel(1);

    let run = Closure {
        lambda: &|this| {
            if let Ok(result) = rx.recv() {
                result(&this);
            }
        },
    };

    thread::spawn(move || gen(tx));
    (run.lambda)(&run);
}

fn main() {
    autoRun(Arc::new(|tx| {
        tx.send(Arc::new(|next| {
            setTimeout(
                &|| {
                    println!("2");
                    (next.lambda)(&next);
                },
                1000,
            );
        }))
        .unwrap();

        tx.send(Arc::new(|next| {
            setTimeout(
                &|| {
                    println!("3");
                    (next.lambda)(&next);
                },
                1000,
            );
        }))
        .unwrap();
    }));
}
