#[test]
fn test_keypair() {
    use zkret_santa_filecoin::crypto::KeyPair;
    let kp = KeyPair::generate();
    let msg = b"hello";
    let sig = kp.sign(msg);
    assert!(kp.verify(msg, &sig));
}
