use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use color_eyre::Report;
use futures::{stream::FuturesUnordered, StreamExt};
use isahc::HttpClient;

pub struct DumbFuture {}

impl Future for DumbFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        Poll::Ready(())
        // panic!("Oh no!");
    }
}

pub async fn test_dumb_future() {
    let future = DumbFuture {};
    future.await;
}

async fn fetch_example_com(client: HttpClient, url: &str) -> Result<http::StatusCode, Report> {
    let res = client.get_async(url).await?;
    let status = res.status();

    Ok(status)
}

pub async fn test_fetch() -> Result<(), Report> {
    let client = HttpClient::builder()
        .build()
        .expect("Failed to create http client");

    let mut group = vec![
        fetch_example_com(client.clone(), "https://example.com"),
        fetch_example_com(client.clone(), "https://example.com"),
    ]
    .into_iter()
    .collect::<FuturesUnordered<_>>();

    while let Some(item) = group.next().await {
        // propagate errors
        item?;
    }
    Ok(())
}

pub async fn test_fetch_2() -> Result<(), Report> {
    let client = HttpClient::builder()
        .build()
        .expect("Failed to create http client");

    let res = tokio::try_join!(
        fetch_example_com(client.clone(), "https://example.com"),
        fetch_example_com(client.clone(), "https://example.com")
    )?;

    Ok(())
}
