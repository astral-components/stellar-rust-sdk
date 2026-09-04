//! Keypair generation and Ed25519 signature tests.

use stellar_rust_sdk::keypair::Keypair;

#[test]
fn generate_import_and_sign_payload() {
    let kp = Keypair::random().expect("rng");
    let seed = kp.secret_seed().unwrap();
    let restored = Keypair::from_secret_seed(seed.as_str()).unwrap();
    assert_eq!(kp.public_key().as_str(), restored.public_key().as_str());

    let payload = b"stellar transaction hash";
    let sig = kp.sign(payload);
    kp.verify(payload, &sig).unwrap();
    restored.verify(payload, &sig).unwrap();
}

#[test]
fn distinct_random_keys() {
    let a = Keypair::random().unwrap();
    let b = Keypair::random().unwrap();
    assert_ne!(a.public_key().as_str(), b.public_key().as_str());
}
