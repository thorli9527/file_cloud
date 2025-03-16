use rand::seq::IteratorRandom;
use std::time::{SystemTime, UNIX_EPOCH};
const UUID_CHARS: &str = "0123456789abcdefghijklmnopqrstuvwxyz";
///
/// 生成
pub fn generate_uuid() -> String {
    let mut rng = rand::rng();
    // 获取纳秒时间戳
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    // 生成 8 位随机字符
    let random_part: String = (0..8)
        .map(|_| UUID_CHARS.chars().choose(&mut rng).unwrap())
        .collect();
    // 组合哈希 IP、时间戳和随机部分
    format!("{}{}", timestamp, random_part)
}

fn main() {
    println!("{}", generate_uuid()); // 生成 10 位随机字符串
}
