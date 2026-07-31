use crate::pretty_print;
use futures::StreamExt;
use governor::clock::{Clock, QuantaClock};
use governor::prelude::StreamRateLimitExt;
use governor::{Quota, RateLimiter};
use reqwest::Client;
use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::num::NonZero;
use std::pin::pin;
use std::time::Duration;

#[derive(Deserialize)]
struct Feed {
    #[serde(rename = "totalResults")]
    total_results: u32,
    #[serde(rename = "startIndex")]
    start_index: u32,
    entry: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    id: String,
}

fn quota() -> Option<Quota> {
    Some(Quota::with_period(Duration::from_secs(1))?.allow_burst(NonZero::new(4)?))
}

fn get_url(base: u32, start: u32, limit: u32) -> String {
    format!(
        "https://export.arxiv.org/api/query?search_query=id:{base}.*&start={start}&max_results={limit}"
    )
}

fn get_src(id: &str) -> String {
    format!("https://arxiv.org/src/{}", id)
}

async fn req_with_backoff_internal(
    client: &Client,
    url: &str,
    max_retries: usize,
    backoff: Duration,
) -> Result<reqwest::Response, Box<dyn Error>> {
    let resp = client.get(url).send().await?;
    if resp.status() == 429 {
        println!("  Got 429, retrying in {} seconds...", backoff.as_secs());
        if max_retries > 0 {
            tokio::time::sleep(backoff).await;
            Box::pin(req_with_backoff_internal(
                client,
                url,
                max_retries - 1,
                backoff * 2,
            ))
            .await
        } else {
            Err(Box::from(format!(
                "Too many retries - couldn't reach {url}"
            )))
        }
    } else {
        Ok(resp)
    }
}

async fn req_with_backoff(
    client: &Client,
    url: &str,
    max_retries: usize,
) -> Result<reqwest::Response, Box<dyn Error>> {
    req_with_backoff_internal(client, url, max_retries, Duration::from_secs(15)).await
}

async fn download(client: &Client, base: u32, id: &str) -> Result<(), Box<dyn Error>> {
    let src = client.get(get_src(id)).send().await?;
    let content_dispo = src
        .headers()
        .get("content-disposition")
        .ok_or_else(|| {
            Box::from("No content-disposition header (paper likely withdrawn)") as Box<dyn Error>
        })?
        .to_str()?
        .to_string();

    let start = content_dispo.find('"').unwrap();
    let filename = &content_dispo[start + 1..content_dispo.len() - 1];

    if filename.ends_with(".pdf") {
        println!("    Skipping PDF-only paper");
        return Ok(());
    }

    let bytes = src.bytes().await?;
    let mut file = File::create(format!("datasets/{base}/{filename}"))?;
    file.write_all(&bytes)?;
    Ok(())
}

pub async fn download_dataset(dataset: u32) -> Result<(), Box<dyn Error>> {
    let quota = quota().unwrap();
    let limiter = RateLimiter::direct(quota);

    let limit = 100;

    let stream = Box::pin(futures::stream::unfold(limit * 85, |state| async move {
        println!("Loading page {}...", state / limit);
        let client = Client::new();
        let result = req_with_backoff(&client, &get_url(dataset, state, limit), 5)
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let feed = quick_xml::de::from_str::<Feed>(&result).unwrap();
        if feed.entry.len() == 0 {
            None
        } else {
            Some((feed, state + limit))
        }
    }))
    .ratelimit_stream(&limiter);

    let mut stream = pin!(stream);

    let client = Client::new();

    fs::create_dir_all(format!("datasets/{dataset}"))?;

    println!("Loading dataset {dataset}...");

    while let Some(feed) = stream.next().await {
        if feed.start_index == 0 {
            println!("Total Results: {}", feed.total_results);
        }
        for entry in feed.entry {
            while let Err(e) = limiter.check() {
                let now = QuantaClock::default().now();
                tokio::time::sleep(e.wait_time_from(now)).await;
            }

            let end = entry.id.rfind('/').unwrap();
            let id = &entry.id[end + 1..];

            println!("  Downloading {id}");

            if let Err(e) = download(&client, dataset, id).await {
                println!("    Error downloading {id}:");
                pretty_print(&*e);
            }
        }
    }

    Ok(())
}
