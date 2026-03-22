use std::time::{SystemTime, UNIX_EPOCH};
use hmac::{Hmac, Mac};
use sha2::{Sha256, Sha512, Digest};
use rand::random;

type HmacSha256 = Hmac<Sha256>;
type HmacSha512 = Hmac<Sha512>;

pub struct SecurityCore {
    device_id: String,
    fingerprint: String,
    primary_key: Vec<u8>,
    secondary_key: Vec<u8>,
    api_key: String,
}

impl SecurityCore {
    pub fn new(api_key: &str) -> Self {
        let device_id = format!("tsd_{}", uuid::Uuid::new_v4().to_string().replace('-', ""));
        let fingerprint = Self::compute_fingerprint(&device_id);
        let primary_key = Self::derive_key(api_key, &device_id, b"tiksync-primary-v1");
        let secondary_key = Self::derive_key(api_key, &device_id, b"tiksync-secondary-v1");

        Self { device_id, fingerprint, primary_key, secondary_key, api_key: api_key.to_string() }
    }

    fn derive_key(api_key: &str, device_id: &str, salt: &[u8]) -> Vec<u8> {
        let mut h = Sha256::new();
        h.update(api_key.as_bytes());
        h.update(b"|");
        h.update(device_id.as_bytes());
        h.update(b"|");
        h.update(salt);
        h.finalize().to_vec()
    }

    fn compute_fingerprint(device_id: &str) -> String {
        let mut h = Sha256::new();
        h.update(device_id.as_bytes());
        h.update(std::env::consts::OS.as_bytes());
        h.update(std::env::consts::ARCH.as_bytes());
        h.update(b"1.0.0");

        let hostname = std::env::var("COMPUTERNAME")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_default();
        h.update(hostname.as_bytes());

        let user = std::env::var("USERNAME")
            .or_else(|_| std::env::var("USER"))
            .unwrap_or_default();
        let mut uh = Sha256::new();
        uh.update(user.as_bytes());
        h.update(hex::encode(uh.finalize()).as_bytes());

        hex::encode(h.finalize())
    }

    pub fn sign(&self, method: &str, path: &str, body: &str) -> (String, String, String) {
        let timestamp = Self::now_ms().to_string();
        let nonce = Self::generate_nonce();

        let payload = format!("{}|{}|{}|{}|{}", method.to_uppercase(), path, &timestamp, &nonce, body);

        let primary_sig = Self::hmac_sha256(&self.primary_key, payload.as_bytes());
        let secondary_input = format!("{}|{}", primary_sig, &nonce);
        let secondary_sig = Self::hmac_sha512(&self.secondary_key, secondary_input.as_bytes());

        let combined = format!("ts1.{}.{}", &primary_sig[..16], &secondary_sig[..24]);
        (combined, timestamp, nonce)
    }

    pub fn generate_token(&self) -> String {
        let epoch = Self::now_ms() / 1000 / 180;
        let nonce_bytes: [u8; 8] = random();
        let nonce_hex = hex::encode(nonce_bytes);

        let payload = format!("{}|{}|{}", epoch, nonce_hex, &self.device_id);
        let master_key = Self::derive_key(&self.api_key, &self.device_id, b"tiksync-token-master-v1");
        let sig = Self::hmac_sha256(&master_key, payload.as_bytes());

        format!("tst1.{}.{}.{}", epoch, nonce_hex, &sig[..32])
    }

    pub fn get_headers(&self) -> Vec<(String, String)> {
        let (sig, ts, nonce) = self.sign("GET", "/v1/connect", "");
        let neural_input = format!("GET|/v1/connect|{}|{}|{}", &ts, &nonce, &self.fingerprint[..32]);
        let neural_sig = self.compute_neural_signature(neural_input.as_bytes());
        let epoch = Self::now_ms() / (6 * 3600 * 1000);

        vec![
            ("x-ts-device".into(), self.device_id.clone()),
            ("x-ts-signature".into(), sig),
            ("x-ts-neural".into(), neural_sig),
            ("x-ts-timestamp".into(), ts),
            ("x-ts-nonce".into(), nonce),
            ("x-ts-token".into(), self.generate_token()),
            ("x-ts-fingerprint".into(), self.fingerprint.clone()),
            ("x-ts-weights".into(), "1".into()),
            ("x-ts-epoch".into(), epoch.to_string()),
            ("x-ts-version".into(), "1.0.0".into()),
        ]
    }

    fn compute_neural_signature(&self, input: &[u8]) -> String {
        let mut h = Sha256::new();
        h.update(input);
        let hash = h.finalize();

        let weights_seed = Self::derive_key(&self.api_key, &self.device_id, b"tiksync-neural-v1");
        let mut output = vec![0u8; 32];
        for i in 0..32 {
            let mut acc: i32 = 0;
            for j in 0..32 {
                acc += (hash[j] as i32) * (weights_seed[(i + j) % 32] as i32);
            }
            let sigmoid = 1.0 / (1.0 + (-(acc as f64 / 8192.0)).exp());
            output[i] = (sigmoid * 255.0) as u8;
        }
        hex::encode(output)
    }

    fn now_ms() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
    }

    fn generate_nonce() -> String {
        let bytes: [u8; 16] = random();
        hex::encode(bytes)
    }

    fn hmac_sha256(key: &[u8], data: &[u8]) -> String {
        let mut mac = HmacSha256::new_from_slice(key).expect("HMAC key");
        mac.update(data);
        hex::encode(mac.finalize().into_bytes())
    }

    fn hmac_sha512(key: &[u8], data: &[u8]) -> String {
        let mut mac = HmacSha512::new_from_slice(key).expect("HMAC key");
        mac.update(data);
        hex::encode(mac.finalize().into_bytes())
    }
}
