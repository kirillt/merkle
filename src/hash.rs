use crypto::digest::Digest;
use crypto::sha2::Sha256;

pub type Key = Vec<u8>;

pub fn hash(bytes: &[u8]) -> Key {
    sha256(&sha256(bytes))
}

#[cfg(test)]
pub fn hash_str(text: &str) -> Key {
    hash(text.as_bytes())
}

pub fn hash_two(left: &[u8], right: &[u8]) -> Key {
    let mut buffer = vec![];
    buffer.extend_from_slice(left);
    buffer.extend_from_slice(right);
    hash(&buffer[..])
}

fn sha256(bytes: &[u8]) -> Key {
    let mut sha = Sha256::new();
    sha.input(bytes);

    let mut out: Key = vec![0u8; sha.output_bytes()];
    sha.result(out.as_mut_slice());
    out
}
