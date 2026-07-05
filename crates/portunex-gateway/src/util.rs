//! 共享工具:ID 生成、随机 token、argon2 口令哈希/校验。
use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use rand::Rng;

const ALPHA: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

/// 占位雪花 ID。TODO: 还原真实雪花(worker_id + 时间戳 + 序列)。
pub fn new_id() -> i64 {
    rand::thread_rng().gen_range(1_000_000_000_000_000..9_000_000_000_000_000)
}
pub fn random_token(n: usize) -> String {
    let mut r = rand::thread_rng();
    (0..n).map(|_| ALPHA[r.gen_range(0..ALPHA.len())] as char).collect()
}
pub fn hash_password(pw: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Ok(Argon2::default()
        .hash_password(pw.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("argon2: {e}"))?
        .to_string())
}
pub fn verify_password(pw: &str, phc: &str) -> bool {
    PasswordHash::new(phc)
        .map(|h| Argon2::default().verify_password(pw.as_bytes(), &h).is_ok())
        .unwrap_or(false)
}
