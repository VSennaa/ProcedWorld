//! Session token generation and at-rest verifiers.
//!
//! The disk (and any log) only ever holds a verifier `sha256$<salt hex>$<digest hex>`, where the
//! digest is SHA-256 over `salt || token` and the salt is 16 random bytes per token. Tokens are 256
//! random bits, so a salted fast hash is enough (they are not human passwords and cannot be
//! dictionary-guessed); the point is that a leaked data directory does not reveal usable tokens.

use sha2::{Digest, Sha256};

const PREFIX: &str = "sha256$";
const SALT_BYTES: usize = 16;
const TOKEN_BYTES: usize = 32;

fn random_bytes<const N: usize>() -> [u8; N] {
    let mut bytes = [0u8; N];
    getrandom::getrandom(&mut bytes).expect("operating system randomness is available");
    bytes
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn from_hex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 || !text.is_ascii() {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).ok())
        .collect()
}

/// A fresh session token: 256 bits from the operating system CSPRNG, hex encoded.
pub fn new_token() -> String {
    to_hex(&random_bytes::<TOKEN_BYTES>())
}

fn digest(salt: &[u8], token: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(salt);
    hasher.update(token.as_bytes());
    hasher.finalize().into()
}

/// Builds the verifier stored on disk for `token`, with a new random salt.
pub fn make_verifier(token: &str) -> String {
    let salt = random_bytes::<SALT_BYTES>();
    format!("{PREFIX}{}${}", to_hex(&salt), to_hex(&digest(&salt, token)))
}

fn parse(stored: &str) -> Option<(Vec<u8>, Vec<u8>)> {
    let (salt, hash) = stored.strip_prefix(PREFIX)?.split_once('$')?;
    let (salt, hash) = (from_hex(salt)?, from_hex(hash)?);
    (salt.len() == SALT_BYTES && hash.len() == 32).then_some((salt, hash))
}

/// True when `stored` is a well-formed verifier (as opposed to a legacy plaintext token).
pub fn is_verifier(stored: &str) -> bool {
    parse(stored).is_some()
}

/// Constant-time check of `token` against a stored verifier.
pub fn verify(token: &str, stored: &str) -> bool {
    let Some((salt, expected)) = parse(stored) else {
        return false;
    };
    let actual = digest(&salt, token);
    actual
        .iter()
        .zip(expected.iter())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verifier_accepts_only_the_original_token_and_never_contains_it() {
        let token = new_token();
        assert_eq!(token.len(), 64);
        let verifier = make_verifier(&token);
        assert!(is_verifier(&verifier));
        assert!(!verifier.contains(&token));
        assert!(verify(&token, &verifier));
        assert!(!verify("other", &verifier));
        assert!(!verify(&token, "not-a-verifier"));
        assert_ne!(make_verifier(&token), verifier, "salt differs per verifier");
        assert!(!is_verifier(&token));
    }
}
