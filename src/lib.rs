use s3::Bucket;
use s3::Region;
use std::vec::Vec;

pub struct Logger {
    bucket: Bucket,
    log_name: String,
    logs: Vec<String>,
}

impl Logger {
    pub fn new(
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
        }
    }

    pub fn log(&mut self, message: &str) {
        println!("{}", message);
        let m = format!("{}\n", message);
        // push to the vector
        self.logs.push(m);
    }

    pub fn flush(&mut self) {
        // download the file
        let resp = self.bucket.get_object_blocking(&self.log_name);
        let file = resp.unwrap();
        // convert the file to a utf8 string
        let mut file_contents = String::new();
        let b = file.bytes();
        for byte in b {
            file_contents.push(*byte as char);
        }
        // append the logs to the file
        file_contents.push_str(&self.logs.join(""));
        // upload the file
        self.bucket
            .put_object_blocking(&self.log_name, file_contents.as_bytes())
            .unwrap();
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
        let logger = Logger::new(
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

        let mut logger = Logger::new(
            getenv("BUCKET"),
            "logs.txt".to_string(),
            Region::UsEast2,
            config,
        );
        logger.log("hello world");
        logger.log("this is a test");
        logger.flush();
    }
}
