use isahc::{prelude::*, HttpClient};

fn test_proxy(port: u16) -> Result<http::status::StatusCode, isahc::Error> {
    let url = format!("http://localhost:{}" , port);

    println!("Testing proxy on url: {}", url);

    let proxy = Some(url.parse().expect("Parse proxy URL error"));
    let client = HttpClient::builder()
        .proxy(proxy)
        .build()
        .expect("Failed to create http client");

    Ok(client.get("https://example.com")?.status())
}

pub fn test_proxy_ports(start: u16, end: u16) {
    let (tx, rx) = crossbeam_channel::unbounded();
    let threads: Vec<_> = (start..end)
        .map(|port| {
            let tx = tx.clone();
            std::thread::spawn(move || {
                if let Ok(status) = test_proxy(port) {
                    tx.send(port).unwrap();
                }
            })
        })
        .collect();


    for thread in threads {
        println!("Joining thread: {:?}", thread.thread().id());
        thread.join().unwrap();
    }

    while let Ok(port) = rx.recv() {
        println!("Avaliable proxy port: {}", port);
        break;
    }
}
