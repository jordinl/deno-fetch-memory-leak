use std::time::{Duration, SystemTime};
use std::fs::File;
use std::env;
use std::io::{self, prelude::*, BufReader};
use reqwest;
use reqwest::header::USER_AGENT;
use tokio;
use futures::prelude::*;
use std::collections::HashMap;
use tokio::task;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use num_format::{Locale, ToFormattedString};
use chrono::{DateTime, Utc};

fn get_env(key: &str, default: u32) -> u32 {
    env::var(key)
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(default)
}

fn log(message: &str) {
    let current_time: DateTime<Utc> = Utc::now();
    println!("[{}] {}", current_time.format("%Y-%m-%dT%H:%M:%S%.3fZ"), message);
}

fn print_memory_usage_and_urls(url_count: &Arc<AtomicUsize>) {
    let memory_usage = psutil::process::Process::new(std::process::id() as u32)
        .unwrap()
        .memory_info()
        .unwrap();
    let memory_usage_mb = memory_usage.rss() as f64 / 1024.0 / 1024.0;
    let processed_urls = url_count.load(Ordering::SeqCst);
    log(&format!(
        "Memory usage: {:.2} MB | Processed URLs: {}",
        memory_usage_mb,
        processed_urls.to_formatted_string(&Locale::en)
    ));
}

async fn periodic_memory_usage(interval: Duration, url_count: Arc<AtomicUsize>) {
    let mut interval = tokio::time::interval(interval);
    loop {
        interval.tick().await;
        print_memory_usage_and_urls(&url_count);
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let url_limit = get_env("LIMIT", 1000);
    let concurrency = get_env("CONCURRENCY", 10);

    log(&format!("Using LIMIT: {:?}",url_limit));
    log(&format!("Using CONCURRENCY: {:?}", concurrency));

    let time = SystemTime::now();
    let url_count = Arc::new(AtomicUsize::new(0));

    let memory_task = {
        let url_count = url_count.clone();
        task::spawn(periodic_memory_usage(Duration::from_secs(10), url_count))
    };

    let file = File::open("urls.txt")?;
    let reader = BufReader::new(file);

    let results: Vec<String> = stream::iter(reader.lines().take(url_limit as usize))
        .map(|line| {
            let url_count = url_count.clone();
            async move {
                let url = line.unwrap();

                let client = reqwest::Client::builder()
                    .timeout(Duration::from_secs(5))
                    .build()
                    .unwrap();

                let response = client.get(&url)
                    .header(USER_AGENT, "crawler-test")
                    .send()
                    .await;

                url_count.fetch_add(1, Ordering::SeqCst);

                match response {
                    Ok(response) => {
                        let status = response.status().as_u16();
                        let status_group = format!("{}XX", status / 100);
                        response.text().await.map_or_else(|_| "error".to_string(), |_| status_group)
                    }
                    Err(_) => "error".to_string(),
                }
            }
        })
        .buffer_unordered(concurrency as usize)
        .collect::<Vec<_>>()
        .await;

    memory_task.abort();


    let mut aggregated_results: HashMap<String, usize> = HashMap::new();
    for result in results {
        *aggregated_results.entry(result).or_insert(0) += 1;
    }

    let mut sorted_results: Vec<(String, usize)> = aggregated_results.into_iter().collect();
    sorted_results.sort_by(|a, b| a.0.cmp(&b.0));

    print_memory_usage_and_urls(&url_count);

    log("Results:");
    for (code, count) in sorted_results {
        log(&format!(" * {}: {}", code, count));
    }

    log(&format!("Total time: {:?}", time.elapsed().unwrap()));

    Ok(())
}