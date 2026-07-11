use std::future::Future;

use futures_util::{StreamExt, stream};

pub(crate) const MAX_MULTI_SYMBOLS: usize = 25;
pub(crate) const MULTI_SYMBOL_CONCURRENCY: usize = 4;

pub(crate) async fn collect_ordered_bounded<I, T, F, Fut>(
    inputs: Vec<I>,
    concurrency: usize,
    operation: F,
) -> Vec<(usize, T)>
where
    F: Fn(usize, I) -> Fut,
    Fut: Future<Output = T>,
{
    assert!(concurrency > 0, "bounded concurrency must be positive");
    let operation = &operation;
    let mut completed = stream::iter(inputs.into_iter().enumerate())
        .map(|(requested_index, input)| async move {
            (requested_index, operation(requested_index, input).await)
        })
        .buffer_unordered(concurrency)
        .collect::<Vec<_>>()
        .await;
    completed.sort_by_key(|(requested_index, _)| *requested_index);
    completed
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
        time::{Duration, Instant, sleep},
    };

    use super::*;

    #[tokio::test]
    async fn bounded_runner_limits_overlap_and_restores_input_order() {
        let active = Arc::new(AtomicUsize::new(0));
        let max_active = Arc::new(AtomicUsize::new(0));
        let inputs = (0usize..25).collect::<Vec<_>>();

        let completed = collect_ordered_bounded(
            inputs,
            MULTI_SYMBOL_CONCURRENCY,
            |requested_index, input| {
                let active = Arc::clone(&active);
                let max_active = Arc::clone(&max_active);
                async move {
                    let now_active = active.fetch_add(1, Ordering::SeqCst) + 1;
                    max_active.fetch_max(now_active, Ordering::SeqCst);
                    sleep(Duration::from_millis((25 - input) as u64)).await;
                    active.fetch_sub(1, Ordering::SeqCst);
                    (requested_index, input)
                }
            },
        )
        .await;

        assert_eq!(active.load(Ordering::SeqCst), 0);
        assert!((2..=MULTI_SYMBOL_CONCURRENCY).contains(&max_active.load(Ordering::SeqCst)));
        assert_eq!(
            completed,
            (0usize..25)
                .map(|index| (index, (index, index)))
                .collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    #[ignore = "deterministic implementation measurement; run explicitly with --ignored --nocapture"]
    async fn measure_sequential_and_bounded_http_workloads() {
        for (workflow, requests_per_symbol) in
            [("quotes", 1usize), ("events_compare", 1), ("compare", 3)]
        {
            let mut results = Vec::new();
            for symbol_count in [1usize, 2, 5, 10, 25] {
                let sequential = median_measurement(symbol_count, requests_per_symbol, 1).await;
                let bounded =
                    median_measurement(symbol_count, requests_per_symbol, MULTI_SYMBOL_CONCURRENCY)
                        .await;
                let improvement = 1.0 - bounded.as_secs_f64() / sequential.as_secs_f64();
                println!(
                    "bounded measurement: workflow={workflow} symbols={symbol_count} sequential_ms={} bounded_ms={} improvement_percent={:.1}",
                    sequential.as_millis(),
                    bounded.as_millis(),
                    improvement * 100.0,
                );
                results.push((symbol_count, improvement));
            }
            for required_count in [10usize, 25] {
                let improvement = results
                    .iter()
                    .find_map(|(count, improvement)| {
                        (*count == required_count).then_some(*improvement)
                    })
                    .unwrap();
                assert!(
                    improvement >= 0.25,
                    "{workflow} did not meet the 25% threshold at {required_count} symbols"
                );
            }
        }
    }

    async fn median_measurement(
        symbol_count: usize,
        requests_per_symbol: usize,
        concurrency: usize,
    ) -> Duration {
        let mut durations = Vec::new();
        for _ in 0..5 {
            durations
                .push(measure_http_workload(symbol_count, requests_per_symbol, concurrency).await);
        }
        durations.sort();
        durations[durations.len() / 2]
    }

    async fn measure_http_workload(
        symbol_count: usize,
        requests_per_symbol: usize,
        concurrency: usize,
    ) -> Duration {
        let request_count = symbol_count * requests_per_symbol;
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let active = Arc::new(AtomicUsize::new(0));
        let max_active = Arc::new(AtomicUsize::new(0));
        let server_active = Arc::clone(&active);
        let server_max_active = Arc::clone(&max_active);
        let server = tokio::spawn(async move {
            let mut tasks = Vec::new();
            for _ in 0..request_count {
                let (mut stream, _) = listener.accept().await.unwrap();
                let active = Arc::clone(&server_active);
                let max_active = Arc::clone(&server_max_active);
                tasks.push(tokio::spawn(async move {
                    let now_active = active.fetch_add(1, Ordering::SeqCst) + 1;
                    max_active.fetch_max(now_active, Ordering::SeqCst);
                    let mut request = Vec::new();
                    let mut buffer = [0u8; 256];
                    loop {
                        let count = stream.read(&mut buffer).await.unwrap();
                        request.extend_from_slice(&buffer[..count]);
                        if request.windows(4).any(|part| part == b"\r\n\r\n") {
                            break;
                        }
                    }
                    sleep(Duration::from_millis(20)).await;
                    stream
                        .write_all(
                            b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n[]",
                        )
                        .await
                        .unwrap();
                    active.fetch_sub(1, Ordering::SeqCst);
                }));
            }
            for task in tasks {
                task.await.unwrap();
            }
        });

        let client = crate::http::configured_client().unwrap();
        let url = format!("http://{address}/measure");
        let started = Instant::now();
        let completed = collect_ordered_bounded(
            (0..symbol_count).collect(),
            concurrency,
            |_, symbol_index| {
                let client = &client;
                let url = &url;
                async move {
                    for _ in 0..requests_per_symbol {
                        client
                            .get(url)
                            .header("X-Synthetic-Symbol", symbol_index)
                            .send()
                            .await
                            .unwrap()
                            .bytes()
                            .await
                            .unwrap();
                    }
                    symbol_index
                }
            },
        )
        .await;
        let elapsed = started.elapsed();
        server.await.unwrap();
        assert_eq!(completed.len(), symbol_count);
        assert_eq!(active.load(Ordering::SeqCst), 0);
        assert!(max_active.load(Ordering::SeqCst) <= concurrency);
        elapsed
    }
}
