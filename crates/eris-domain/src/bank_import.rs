use anyhow::Result;
use sha2::Sha256;
use sqlx::FromRow;


/// hash_iban takes an iban as string and name as string
/// and creates the hash by using the 12 first bytes of the hextdigest of
/// the pbkdf2_hmac function with sha256, using the name and iban as salt
pub fn hash_iban(iban: &str, name: &str) -> String {
    // Derive key using pbkdf2 hmac from name and iban
    let mut key = [0u8; 6];
    let name_bytes = name.as_bytes();
    let iban_bytes = iban.as_bytes();

    pbkdf2::pbkdf2_hmac::<Sha256>(name_bytes, iban_bytes, 1000, &mut key);
    // Hexdigest the key
    let hash = hex::encode(key);
    hash
}

#[derive(Debug, Default, Clone)]
pub struct BankImportRuleFilter {
    pub member_id: Option<u32>,
    pub iban_hash: Option<String>,
}

#[derive(Debug, Clone, Default, FromRow)]
pub struct BankImportRule {
    pub member_id: u32,
    pub iban: String,
    pub split_amount: Option<f64>,
    pub match_subject: Option<String>,
}

