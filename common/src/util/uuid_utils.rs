use uuid::Uuid;

pub fn build_id() -> String {
    let uuid = Uuid::new_v4();
    return uuid.simple().to_string();
}
