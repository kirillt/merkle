use crypto::digest::Digest;
use crypto::sha2::Sha256;

pub fn hash(value: &str) -> String {
    sha256(&sha256(value))
}

fn sha256(value: &str) -> String {
    let mut sha = Sha256::new();
    sha.input_str(value);
    sha.result_str()
}
