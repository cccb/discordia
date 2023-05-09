use anyhow::Result;
use sha2::Sha256;
use sqlx::{FromRow, QueryBuilder, Sqlite};

use crate::{database::Database, db::Error};

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
pub struct MemberIbanFilter {
    pub id: Option<u32>,
    pub iban_hash: Option<String>,
}

#[derive(Debug, Clone, Default, FromRow)]
pub struct BankImportMemberIban {
    pub member_id: u32,
    pub iban_hash: String,
    pub split_amount: Option<f64>,
}

impl BankImportMemberIban {
    /// Fetch member IBANs
    pub async fn filter(
        db: &Database,
        filter: Option<MemberIbanFilter>,
    ) -> Result<Vec<BankImportMemberIban>> {
        let mut conn = db.lock().await;
        let mut qry = QueryBuilder::new(
            r#"
            SELECT 
                member_id,
                iban_hash,
                ROUND(split_amount, 10) AS split_amount
            FROM bank_import_member_ibans
            WHERE 1
            "#,
        );
        if let Some(filter) = filter {
            if let Some(id) = filter.id {
                qry.push(" AND member_id = ").push_bind(id);
            }
            if let Some(iban_hash) = filter.iban_hash {
                qry.push(" AND iban_hash = ").push_bind(iban_hash);
            }
        }
        let ibans: Vec<BankImportMemberIban> = qry.build_query_as().fetch_all(&mut *conn).await?;
        Ok(ibans)
    }

    // Get a single member IBAN
    pub async fn get(
        db: &Database,
        member_id: u32,
        iban_hash: &str,
    ) -> Result<BankImportMemberIban> {
        let filter = MemberIbanFilter {
            id: Some(member_id),
            iban_hash: Some(iban_hash.to_string()),
        };
        let ibans = Self::filter(db, Some(filter)).await?;
        if ibans.len() == 0 {
            return Err(Error::NotFound.into());
        }
        if ibans.len() > 1 {
            return Err(Error::Ambiguous(ibans.len()).into());
        }
        Ok(ibans[0].clone())
    }

    /// Update member IBAN
    pub async fn update(&self, db: &Database) -> Result<BankImportMemberIban> {
        {
            let mut conn = db.lock().await;
            QueryBuilder::<Sqlite>::new("UPDATE bank_import_member_ibans SET")
                .push(" split_amount = ")
                .push_bind(format!("{}", self.split_amount.unwrap_or(0.0)))
                .push(" WHERE member_id = ")
                .push_bind(self.member_id)
                .push(" AND iban_hash = ")
                .push_bind(&self.iban_hash)
                .build()
                .execute(&mut *conn)
                .await?;
        }
        Self::get(db, self.member_id, &self.iban_hash).await
    }

    /// Create member IBAN
    pub async fn insert(&self, db: &Database) -> Result<BankImportMemberIban> {
        {
            let mut conn = db.lock().await;
            let mut qry = QueryBuilder::<Sqlite>::new(
                r#"INSERT INTO bank_import_member_ibans (
                    member_id,
                    iban_hash,
            "#,
            );
            if self.split_amount.is_some() {
                qry.push(" split_amount ");
            }
            qry.push(" ) VALUES ( ");
            qry.separated(", ")
                .push_bind(self.member_id)
                .push_bind(&self.iban_hash);
            if let Some(split_amount) = self.split_amount {
                qry.push_bind(format!("{}", split_amount));
            }
            qry.push(") ");
            qry.build().execute(&mut *conn).await?;
        }
        Self::get(db, self.member_id, &self.iban_hash).await
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
}
