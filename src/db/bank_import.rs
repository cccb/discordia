use anyhow::Result;
use sha2::Sha256;
use sqlx::{FromRow, QueryBuilder, Sqlite};

use crate::{
    db::{
        Error,
        members::Member,
        connection::Connection,
}};

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
    pub match_subject: Option<String>,
}

impl BankImportMemberIban {
    /// Fetch member IBANs
    pub async fn filter(
        db: &Connection,
        filter: &MemberIbanFilter,
    ) -> Result<Vec<BankImportMemberIban>> {
        let mut conn = db.lock().await;
        let mut qry = QueryBuilder::new(
            r#"
            SELECT 
                member_id,
                iban_hash,
                match_subject,
                ROUND(split_amount, 10) AS split_amount
            FROM bank_import_member_ibans
            WHERE 1
            "#,
        );
        if let Some(id) = filter.id {
            qry.push(" AND member_id = ").push_bind(id);
        }
        if let Some(iban_hash) = filter.iban_hash.clone() {
            qry.push(" AND iban_hash = ").push_bind(iban_hash);
        }
        let ibans: Vec<BankImportMemberIban> = qry.build_query_as().fetch_all(&mut *conn).await?;
        Ok(ibans)
    }

    // Get a single member IBAN
    pub async fn get(
        db: &Connection,
        member_id: u32,
        iban_hash: &str,
    ) -> Result<BankImportMemberIban> {
        let filter = MemberIbanFilter {
            id: Some(member_id),
            iban_hash: Some(iban_hash.to_string()),
        };
        let ibans = Self::filter(db, &filter).await?;
        if ibans.len() == 0 {
            return Err(Error::NotFound.into());
        }
        if ibans.len() > 1 {
            return Err(Error::Ambiguous(ibans.len()).into());
        }
        Ok(ibans[0].clone())
    }

    /// Update member IBAN
    pub async fn update(&self, db: &Connection) -> Result<BankImportMemberIban> {
        {
            let mut conn = db.lock().await;
            let mut split_amount: Option<String> = None;
            if let Some(amount) = self.split_amount {
                split_amount = Some(format!("{}", amount)); 
            }

            QueryBuilder::<Sqlite>::new("UPDATE bank_import_member_ibans SET")
                .push(" split_amount = ")
                .push_bind(&split_amount)
                .push(", match_subject = ")
                .push_bind(&self.match_subject)
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
    pub async fn insert(&self, db: &Connection) -> Result<BankImportMemberIban> {
        {
            let mut split_amount: Option<String> = None;
            if let Some(amount) = self.split_amount {
                split_amount = Some(format!("{}", amount)); 
            }

            let mut conn = db.lock().await;
            let mut qry = QueryBuilder::<Sqlite>::new(
                r#"INSERT INTO bank_import_member_ibans (
                    member_id,
                    iban_hash,
                    match_subject,
                    split_amount
            "#,
            );
            qry.push(" ) VALUES ( ");
            qry.separated(", ")
                .push_bind(self.member_id)
                .push_bind(&self.iban_hash)
                .push_bind(&self.match_subject)
                .push_bind(&split_amount);
            qry.push(") ");
            qry.build().execute(&mut *conn).await?;
        }
        Self::get(db, self.member_id, &self.iban_hash).await
    }

    /// Delete a member IBAN rule
    pub async fn delete(&self, db: &Connection) -> Result<()> {
        let mut conn = db.lock().await;
        QueryBuilder::<Sqlite>::new("DELETE FROM bank_import_member_ibans WHERE")
            .push(" member_id = ")
            .push_bind(self.member_id)
            .push(" AND iban_hash = ")
            .push_bind(&self.iban_hash)
            .build()
            .execute(&mut *conn)
            .await?;

        Ok(())
    }

    /// Get associated member
    pub async fn member(&self, db: &Connection) -> Result<Member> {
        Member::get(db, self.member_id).await
    }

    /// Match a transaction subjec: If the match_subject is a
    /// substring of the transaction subject it will be true.
    /// Comparison is case insensitive.
    pub fn match_subject(&self, subject: &str) -> bool {
        if self.match_subject == None {
            return false
        }
        let match_subject = self.match_subject.clone().unwrap();
        let subject = subject.clone().to_lowercase();
        
        subject.contains(&match_subject)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection;

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
        let rule = BankImportMemberIban{
            match_subject: Some("beitrag".to_string()),
            ..Default::default()
        };
        assert!(rule.match_subject("Mitgliedsbeitrag 2024"));
        assert!(rule.match_subject("Beitrag fuer maerz"));
        assert!(!rule.match_subject("Sonstiges"));
    }

    #[tokio::test]
    async fn test_bank_import_member_iban_insert() {
        let (_handle, conn) = connection::open_test().await;

        let m = Member{
            name: "Testmember1".to_string(),
            ..Member::default()
        };
        let m = m.insert(&conn).await.unwrap();

        let rule = BankImportMemberIban{
            member_id: m.id,
            iban_hash: "hash".to_string(),
            split_amount: None,
            match_subject: Some("beitrag".to_string()),
        };
        let rule = rule.insert(&conn).await.unwrap();
        assert_eq!(rule.member_id, m.id);
        assert_eq!(rule.iban_hash, "hash");
        assert_eq!(rule.match_subject, Some("beitrag".to_string()));
    }

    #[tokio::test]
    async fn test_bank_import_member_iban_update() {
        let (_handle, conn) = connection::open_test().await;
        let m = Member{
            name: "Testmember1".to_string(),
            ..Member::default()
        };
        let m = m.insert(&conn).await.unwrap();

        let rule = BankImportMemberIban{
            member_id: m.id,
            iban_hash: "hash".to_string(),
            split_amount: Some(23.42),
            match_subject: None,
        };
        let mut rule = rule.insert(&conn).await.unwrap();

        assert_eq!(rule.match_subject, None);
        assert_eq!(rule.split_amount, Some(23.42));

        // Update rule
        rule.match_subject = Some("beitrag".to_string());
        rule.split_amount = None;

        let rule = rule.update(&conn).await.unwrap();

        assert_eq!(rule.member_id, m.id);
        assert_eq!(rule.match_subject, Some("beitrag".to_string()));
        assert_eq!(rule.split_amount, None);
    }

    #[tokio::test]
    async fn test_bank_import_member_iban_delete() {
        let (_handle, conn) = connection::open_test().await;
        let m = Member{
            name: "Testmember1".to_string(),
            ..Member::default()
        };
        let m = m.insert(&conn).await.unwrap();

        let rule = BankImportMemberIban{
            member_id: m.id,
            iban_hash: "hash".to_string(),
            split_amount: Some(23.42),
            match_subject: None,
        };
        let rule = rule.insert(&conn).await.unwrap();

        rule.delete(&conn).await.unwrap();
    }
}
