# S3Logger for Rust

This is a super simple logging library for Rust that prints logs to console as well as uploads those logs to S3 as plain text.

# Example

```rust
use s3::Credentials;
use s3::Region;
use s3logger::Logger;

use std::env;

fn main() {
    let logger = Logger::new_blocking(
        "my-bucket",
        "my-logs.txt",
        Region::UsEast1,
        Credentials::from_env()::unwrap(),
    );

    logger.log("hello world!");
    logger.log("This is some text");
    logger.flush_blocking();
}

async fn main_async() {
    let logger = Logger::new(
        "my-bucket",
        "my-logs.txt",
        Region::UsEast1,
        Credentials::from_env()::unwrap(),
    ).await;

    logger.log("Async and sync both use 'log'");
    logger.log("The only difference is the 'new' and 'flush' functions");
    logger.flush().await;
}
```
