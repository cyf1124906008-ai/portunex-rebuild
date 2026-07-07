//! 自建 OIDC 提供方的 JWT 签名基础:启动时生成 RSA 密钥对,提供 JWKS 与 id_token 签发。
use std::sync::OnceLock;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use rsa::pkcs8::EncodePrivateKey;
use rsa::traits::PublicKeyParts;
use rsa::{RsaPrivateKey, RsaPublicKey};
use serde::Serialize;
use serde_json::{json, Value};

pub const KID: &str = "portunex-oidc-1";

struct Keys { pem: String, n_b64: String, e_b64: String }
static KEYS: OnceLock<Keys> = OnceLock::new();

fn keys() -> &'static Keys {
    KEYS.get_or_init(|| {
        // 注:每次启动生成新密钥(演示/重建版)。生产应持久化到 DB/文件。
        let mut rng = rand::thread_rng();
        let priv_key = RsaPrivateKey::new(&mut rng, 2048).expect("rsa keygen");
        let pem = priv_key.to_pkcs8_pem(rsa::pkcs8::LineEnding::LF).expect("pem").to_string();
        let pub_key = RsaPublicKey::from(&priv_key);
        let n_b64 = URL_SAFE_NO_PAD.encode(pub_key.n().to_bytes_be());
        let e_b64 = URL_SAFE_NO_PAD.encode(pub_key.e().to_bytes_be());
        Keys { pem, n_b64, e_b64 }
    })
}

/// 发布 JWKS(公钥,供依赖方验签)。
pub fn jwks() -> Value {
    let k = keys();
    json!({ "keys": [{
        "kty": "RSA", "use": "sig", "alg": "RS256", "kid": KID,
        "n": k.n_b64, "e": k.e_b64
    }]})
}

#[derive(Serialize)]
struct IdClaims<'a> {
    iss: &'a str, sub: String, aud: &'a str,
    email: Option<String>, exp: usize, iat: usize,
}

/// 签发 RS256 id_token。
pub fn sign_id_token(issuer: &str, subject: i64, audience: &str, email: Option<String>, ttl_secs: usize, now: usize) -> anyhow::Result<String> {
    let claims = IdClaims {
        iss: issuer, sub: subject.to_string(), aud: audience,
        email, iat: now, exp: now + ttl_secs,
    };
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(KID.to_string());
    let key = EncodingKey::from_rsa_pem(keys().pem.as_bytes())?;
    Ok(encode(&header, &claims, &key)?)
}
