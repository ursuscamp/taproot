use bitcoin::XOnlyPublicKey;
use sha2::Digest;

static NUMS_INPUT: &'static str = "hello world";

fn main() {
    let nums_point = nums_point();
    print!("NUMS Point: {nums_point}");
}

/// Calculate the NUMS point for a keyless taproot spend.
fn nums_point() -> XOnlyPublicKey {
    let mut hashed = sha2::Sha256::digest(NUMS_INPUT.as_bytes());
    let mut pk = XOnlyPublicKey::from_slice(hashed.as_slice()).ok();

    while pk.is_none() {
        hashed = sha2::Sha256::digest(hashed.as_slice());
        pk = XOnlyPublicKey::from_slice(hashed.as_slice()).ok();
    }
    pk.unwrap()
}
