# Yield Page

[![Rust](https://github.com/USER_OR_ORG/yield-page/workflows/Rust/badge.svg)](https://github.com/USER_OR_ORG/yield-page/actions)
[![codecov](https://codecov.io/gh/USER_OR_ORG/yield-page/branch/main/graph/badge.svg)](https://codecov.io/gh/USER_OR_ORG/yield-page)

A Rust-based web crawler that extracts page data using WebDriver.

## Features

- Concurrent web crawling with configurable parallelism
- URL filtering with regex patterns
- HTML and text parsing
- Configurable request parameters

## Getting Started

### Prerequisites

- Rust toolchain
- WebDriver (e.g., ChromeDriver, GeckoDriver)

### Usage

```bash
# Run with default settings
cargo run -- crawl https://example.com

# Run with custom config
cargo run -- crawl --config my_config.json https://example.com
```

## Configuration

See `example_config.json` for configuration options.

## Development

```bash
# Run tests
cargo test

# Run clippy lints
cargo clippy

# Generate code coverage
cargo install cargo-tarpaulin
cargo tarpaulin
```
