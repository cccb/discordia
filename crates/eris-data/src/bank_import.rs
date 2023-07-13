use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sqlx::FromRow;

use crate::{Member, Retrieve};

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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BankImportRuleFilter {
    pub member_id: Option<u32>,
    pub iban: Option<String>,
}

#[derive(Debug, Clone, Default, FromRow, Serialize, Deserialize)]
pub struct BankImportRule {
    pub member_id: u32,
    pub iban: String,
    pub split_amount: Option<f64>,
    pub match_subject: Option<String>,
}


impl BankImportRule {

    /// Create a new standard rule with no split and
    /// subject match for a member and iban
    pub fn new(member: &Member, iban: &str) -> Self {
        Self {
            member_id: member.id,
            iban: iban.to_string(),
            ..Default::default()
        }
    }

    /// Get associated member
    pub async fn get_member<DB>(&self, db: &DB) -> Result<Member>
    where
        DB: Retrieve<Member, Key=u32>,
    {
        db.retrieve(self.member_id).await
    }


    /// Match a transaction subjec: If the match_subject is a
    /// substring of the transaction subject it will be true.
    /// Comparison is case insensitive.
    pub fn match_subject(&self, subject: &str) -> Option<bool> {
        if self.match_subject.is_none() {
            return None
        }
        let match_subject = self.match_subject.clone().unwrap();
        let subject = subject.clone().to_lowercase();
        
        Some(subject.contains(&match_subject))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_iban() {
        let iban = "DE12345678901234567890";
        let name = "Eris Discordia";
        let hash = hash_iban(iban, name);
        assert_eq!(hash, "448a2be23338");
        assert_eq!(hash.len(), 12);
    }

    #[test]
    fn test_match_subject() {
        let rule = BankImportRule{
            match_subject: Some("beitrag".to_string()),
            ..Default::default()
        };
        assert!(rule.match_subject("Mitgliedsbeitrag 2024").unwrap());
        assert!(rule.match_subject("Beitrag fuer maerz").unwrap());
        assert!(!rule.match_subject("Sonstiges").unwrap());
    }

    #[test]
    fn test_match_subject_none() {
        let rule = BankImportRule::default();
        assert_eq!(rule.match_subject("Mitgliedsbeitrag 2024"), None);
    }
}
