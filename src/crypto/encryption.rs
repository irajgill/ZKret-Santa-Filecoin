pub fn encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter().zip(key.iter().cycle()).map(|(a, b)| a ^ b).collect()
}
