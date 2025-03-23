use hex::encode;
use md5;
use md5::{Digest, Md5};

pub fn build_md5(content: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(content);
    let result = hasher.finalize();
    let hex_string = encode(result);
    hex_string
}
