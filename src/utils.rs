use std::time::Duration;

/// Utility function to create a reasonable timeout for web requests
pub fn calculate_timeout(base_ms: u64, url_length: usize) -> Duration {
    // Add additional time for longer URLs which might be more complex to process
    let additional_ms = (url_length / 20) as u64 * 100;
    Duration::from_millis(base_ms + additional_ms)
}

/// Convert a string to a sanitized filename
pub fn sanitize_filename(url: &str) -> String {
    // Remove protocol and replace invalid filename characters
    let mut name = url.replace("http://", "").replace("https://", "");
    name = name.replace(['/', ':', '?', '&', '=', '#', '%'], "_");

    // Limit filename length
    if name.len() > 100 {
        name[..100].to_string()
    } else {
        name
    }
}
