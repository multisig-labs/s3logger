use chrono::Local;
use s3::Bucket;
use s3::Region;
use std::fs::OpenOptions;
use std::io::Write;
use std::vec::Vec;

#[derive(PartialEq)]
pub enum TimestampMode {
    Flush,
    Log,
    None,
}

// Logger
pub struct Logger {
    bucket: Bucket,
    log_name: String,
    logs: Vec<String>,
    timestamp_mode: TimestampMode,
    local_copy: bool,
}

fn append_to_file(contents: String, name: String) {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(name)
        .unwrap();

    file.write(contents.as_bytes()).unwrap();
}

impl Logger {
    // asynchronous function to create a new logger
    pub async fn new(
        bucket: String,
        log_name: String,
        region: Region,
        credentials: s3::creds::Credentials,
    ) -> Self {
        let bucket = Bucket::new(&bucket, region, credentials).unwrap();
        // check to see if the object exists in the bucket
        let log_exists = bucket.head_object(&log_name).await.is_ok();
        if !log_exists {
            // create the object if it doesn't exist
            bucket.put_object(&log_name, b"").await.unwrap();
        }
        Self {
            bucket,
            log_name,
            logs: Vec::new(),
            timestamp_mode: TimestampMode::Log,
            local_copy: false,
        }
    }

    // blocking function to create a new logger
    pub fn new_blocking(
        bucket: String,
        log_name: String,
        region: Region,
        credentials: s3::creds::Credentials,
    ) -> Self {
        let bucket = Bucket::new(&bucket, region, credentials).unwrap();
        // check to see if the object exists in the bucket
        let log_exists = bucket.head_object_blocking(&log_name).is_ok();
        if !log_exists {
            // create the object if it doesn't exist
            bucket.put_object_blocking(&log_name, b"").unwrap();
        }
        Self {
            bucket,
            log_name,
            logs: Vec::new(),
            timestamp_mode: TimestampMode::None,
            local_copy: false,
        }
    }

    pub fn set_timestamp_mode(&mut self, timestamp_mode: TimestampMode) {
        self.timestamp_mode = timestamp_mode;
    }

    pub fn save_local_copy(&mut self, local_copy: bool) {
        self.local_copy = local_copy;
    }

    // logs to the logger. Does not write to the bucket.
    pub fn log(&mut self, message: &str) {
        let m: String;
        if self.timestamp_mode == TimestampMode::Log {
            m = format!("{}: {}\n", Local::now(), message);
        } else {
            m = format!("{}\n", message);
        }
        print!("{}", m);
        // push to the vector
        self.logs.push(m);
    }

    // asynchronous function to write the logs to the bucket
    pub async fn flush(&mut self) {
        // download the file
        let resp = self.bucket.get_object(&self.log_name).await;
        let file = resp.unwrap();
        // convert the file to a utf8 string
        let mut file_contents = String::new();
        let b = file.bytes();
        for byte in b {
            file_contents.push(*byte as char);
        }

        if self.timestamp_mode == TimestampMode::Flush {
            // add a timestamp to the beginning of the file
            file_contents = format!("{}\n", Local::now().format("%Y-%m-%d %H:%M:%S"));
        }
        // append the logs to the file
        file_contents.push_str(&self.logs.join(""));
        // upload the file
        self.bucket
            .put_object(&self.log_name, file_contents.as_bytes())
            .await
            .unwrap();

        if self.local_copy {
            append_to_file(file_contents, self.log_name.clone());
        }

        // clear the logs
        self.logs.clear();
    }

    // blocking function to write the logs to the bucket
    pub fn flush_blocking(&mut self) {
        // download the file
        let resp = self.bucket.get_object_blocking(&self.log_name);
        let file = resp.unwrap();
        // convert the file to a utf8 string
        let mut file_contents = String::new();
        let b = file.bytes();
        for byte in b {
            file_contents.push(*byte as char);
        }

        if self.timestamp_mode == TimestampMode::Flush {
            // add a timestamp to the beginning of the file
            file_contents = format!("{}\n", Local::now().format("%Y-%m-%d %H:%M:%S"));
        }

        // append the logs to the file
        file_contents.push_str(&self.logs.join(""));
        // upload the file
        self.bucket
            .put_object_blocking(&self.log_name, file_contents.as_bytes())
            .unwrap();

        if self.local_copy {
            append_to_file(file_contents, self.log_name.clone());
        }

        // clear the logs
        self.logs.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn getenv(key: &str) -> String {
        match env::var(key) {
            Ok(val) => val,
            Err(_) => panic!("{} is not set", key),
        }
    }

    #[test]
    fn constructor() {
        let logger = Logger::new_blocking(
            getenv("BUCKET"),
            "logs.txt".to_string(),
            Region::UsEast2,
            s3::creds::Credentials::from_env().unwrap(),
        );
        assert_eq!(logger.log_name, "logs.txt");
    }

    #[test]
    fn log() {
        let config = s3::creds::Credentials::from_env().unwrap();

        let mut logger = Logger::new_blocking(
            getenv("BUCKET"),
            "logs.txt".to_string(),
            Region::UsEast2,
            config,
        );
        logger.log("hello world");
        logger.log("this is a test");
        logger.flush_blocking();
    }

    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    #[test]
    fn log_async() {
        // sleep for a few seconds between tests
        std::thread::sleep(std::time::Duration::from_secs(1));
        let config = s3::creds::Credentials::from_env().unwrap();

        let mut logger = aw!(Logger::new(
            getenv("BUCKET"),
            "logs.txt".to_string(),
            Region::UsEast2,
            config,
        ));
        logger.log("hello world");
        logger.log("this is a test");
        aw!(logger.flush());
    }

    #[test]
    fn log_mode_test() {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let config = s3::creds::Credentials::from_env().unwrap();

        let mut logger = Logger::new_blocking(
            getenv("BUCKET"),
            "logs.txt".to_string(),
            Region::UsEast2,
            config,
        );

        logger.set_timestamp_mode(TimestampMode::None);

        logger.log("hello world");
        logger.log("this is a test");

        logger.set_timestamp_mode(TimestampMode::Flush);
        logger.save_local_copy(true);

        logger.log("hello world");
        logger.log("this is a test");

        logger.flush_blocking();
    }
}
