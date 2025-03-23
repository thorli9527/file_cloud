use uuid::Uuid;

pub fn build_id() -> String {
    let uuid = Uuid::new_v4().simple();
    format!("{}", uuid)
}

