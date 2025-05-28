pub fn serialize<T: serde::Serialize>(item: &T) -> Vec<u8> {
    bincode::serialize(item).unwrap()
}
