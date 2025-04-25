use base64::{Engine as _, engine::general_purpose};
use sha2::{Sha256, Digest};

pub fn verify_code_challenge(
    code_verifier: &str,
    code_challenge: &str,
    code_challenge_method: &str,
) -> Result<(), &'static str> {
    match code_challenge_method.to_lowercase().as_str() {
        "s256" => {
            let mut hasher = Sha256::new();
            hasher.update(code_verifier.as_bytes());
            let hashed = hasher.finalize();
            let encoded = general_purpose::URL_SAFE_NO_PAD.encode(hashed);
            
            if encoded == code_challenge {
                Ok(())
            } else {
                Err("Invalid code verifier")
            }
        },
        "plain" => {
            if code_verifier == code_challenge {
                Ok(())
            } else {
                Err("Invalid code verifier")
            }
        },
        _ => Err("Unsupported code challenge method"),
    }
}