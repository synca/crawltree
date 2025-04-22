use crawltree::start_crawler;

#[tokio::main]
async fn main() {
    let start_url = "https://envoyproxy.io/docs/envoy/latest";
    let concurrency = 4;

    let mut results = start_crawler(start_url, concurrency).await;

    while let Some(page) = results.recv().await {
        println!("RX: {}", page.url);
        // Only print first 100 chars of content to avoid flooding logs
        let content_preview = if page.content.len() > 100 {
            format!("{}...", &page.content[..100])
        } else {
            page.content.clone()
        };
        println!("RX content: {}", content_preview);
        // You can embed/store/process here in real time
    }
}
