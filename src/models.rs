use anyhow::Result;
use chrono::NaiveDate;
use sha2::Sha256;
use sqlx::{FromRow, QueryBuilder, Sqlite};

use crate::db::Connection;

/// Model errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("Not found")]
    NotFound,
    #[error("Ambiguous results ({0:?}) for query")]
    Ambiguous(usize),
}

#[derive(Debug, Clone, FromRow)]
pub struct Insert {
    pub id: u32,
}

#[derive(Debug, Clone, FromRow)]
pub struct InsertString {
    pub id: String,
}

#[derive(Debug, Clone, Default, FromRow)]
pub struct Member {
    pub id: u32,
    pub name: String,
    pub email: String,
    pub notes: String,
    pub membership_start: NaiveDate,
    pub membership_end: Option<NaiveDate>,
    pub fee: f64,
    pub interval: u8,
    pub last_payment: NaiveDate,
    pub account: f64,
}

#[derive(Debug, Default, Clone)]
pub struct MemberFilter {
    pub id: Option<u32>,
    pub name: Option<String>,
    pub email: Option<String>,
}

impl Member {
    /// Build query
    fn query<'q>(filter: Option<MemberFilter>) -> QueryBuilder<'q, Sqlite> {
        let mut qry = QueryBuilder::new(
            r#"
            SELECT 
                id,
                name,
                email,
                notes,
                membership_start,
                membership_end,
                last_payment,
                interval,
                ROUND(fee, 10) AS fee,
                ROUND(account, 10) AS account
            FROM members
            WHERE 1
            "#,
        );
        if let Some(filter) = filter {
            if let Some(id) = filter.id {
                qry.push(" AND id = ").push_bind(id);
            }
            if let Some(name) = filter.name {
                qry.push(" AND name LIKE ").push_bind(format!("%{}%", name));
            }
            if let Some(email) = filter.email {
                qry.push(" AND email LIKE ").push_bind(email);
            }
        }
        qry
    }

    /// Fetch members
    pub async fn filter(conn: &Connection, filter: Option<MemberFilter>) -> Result<Vec<Member>> {
        let mut conn = conn.lock().await;
        let members: Vec<Member> = Self::query(filter)
            .build_query_as()
            .fetch_all(&mut *conn)
            .await?;
        Ok(members)
    }

    /// Fetch a single member by ID
    pub async fn get(db: &Database, id: u32) -> Result<Member> {
        let mut conn = db.lock().await;
        let filter = MemberFilter {
            id: Some(id),
            ..MemberFilter::default()
        };
        let member: Member = Self::query(Some(filter))
            .build_query_as()
            .fetch_one(&mut *conn)
            .await?;
        Ok(member)
    }

    /// Update member
    pub async fn update(&self, db: &Database) -> Result<Member> {
        {
            let mut conn = db.lock().await;
            QueryBuilder::<Sqlite>::new("UPDATE members SET")
                .push(" name = ")
                .push_bind(&self.name)
                .push(", email = ")
                .push_bind(&self.email)
                .push(", notes = ")
                .push_bind(&self.notes)
                .push(", membership_start = ")
                .push_bind(self.membership_start)
                .push(", membership_end = ")
                .push_bind(self.membership_end)
                .push(", last_payment = ")
                .push_bind(self.last_payment)
                .push(", interval = ")
                .push_bind(self.interval)
                .push(", fee = ")
                .push_bind(format!("{}", self.fee))
                .push(", account = ")
                .push_bind(format!("{}", self.account))
                .push(" WHERE id = ")
                .push_bind(self.id)
                .build()
                .execute(&mut *conn)
                .await?;
        }
        Self::get(db, self.id).await
    }

    /// Create member
    pub async fn insert(&self, db: &Database) -> Result<Member> {
        let insert: Insert = {
            let mut conn = db.lock().await;
            let mut qry = QueryBuilder::<Sqlite>::new(
                r#"INSERT INTO members (
                    name,
                    email,
                    notes,
                    membership_start,
                    membership_end,
                    last_payment,
                    interval,
                    fee,
                    account
                ) VALUES (
                "#,
            );
            qry.separated(", ")
                .push_bind(&self.name)
                .push_bind(&self.email)
                .push_bind(&self.notes)
                .push_bind(self.membership_start)
                .push_bind(self.membership_end)
                .push_bind(self.last_payment)
                .push_bind(self.interval)
                .push_bind(format!("{}", self.fee))
                .push_bind(format!("{}", self.account));

            qry.push(") RETURNING id ")
                .build_query_as()
                .fetch_one(&mut *conn)
                .await?
        };
        Self::get(db, insert.id).await
    }
}

#[derive(Debug, Default, Clone)]
pub struct TransactionFilter {
    pub member_id: Option<u32>,
    pub date: Option<NaiveDate>,
    pub date_before: Option<NaiveDate>,
    pub date_after: Option<NaiveDate>,
}

#[derive(Debug, Default, Clone, FromRow)]
pub struct Transaction {
    pub id: u32,
    pub member_id: u32,
    pub date: NaiveDate,
    pub account_name: String,
    pub amount: f64,
    pub description: String,
}

impl Transaction {
    // Filter transactions
    pub async fn filter(
        db: &Database,
        filter: Option<TransactionFilter>,
    ) -> Result<Vec<Transaction>> {
        let mut conn = db.lock().await;
        let mut qry = QueryBuilder::<Sqlite>::new(
            r#"
            SELECT 
                id,
                member_id,
                date,
                account_name,
                ROUND(amount, 10) AS amount,
                description
            FROM transactions
            WHERE 1
            "#,
        );
        if let Some(filter) = filter {
            if let Some(member_id) = filter.member_id {
                qry.push(" AND member_id = ").push_bind(member_id);
            }
            if let Some(date) = filter.date {
                qry.push(" AND date = ").push_bind(date);
            }
            if let Some(date_before) = filter.date_before {
                qry.push(" AND date <= ").push_bind(date_before);
            }
            if let Some(date_after) = filter.date_after {
                qry.push(" AND date >= ").push_bind(date_after);
            }
        }
        let transactions: Vec<Transaction> = qry.build_query_as().fetch_all(&mut *conn).await?;
        Ok(transactions)
    }

    /// Fetch a single transaction by ID
    pub async fn get(db: &Database, id: u32) -> Result<Transaction> {
        let filter = TransactionFilter {
            member_id: Some(id),
            ..TransactionFilter::default()
        };
        let transaction: Transaction = Self::filter(db, Some(filter))
            .await?
            .pop()
            .ok_or_else(|| Error::NotFound)?;
        Ok(transaction)
    }

    /// Update a transaction
    pub async fn update(&self, db: &Database) -> Result<Transaction> {
        {
            let mut conn = db.lock().await;
            QueryBuilder::<Sqlite>::new("UPDATE transactions SET")
                .push(" member_id = ")
                .push_bind(self.member_id)
                .push(", date = ")
                .push_bind(self.date)
                .push(", account_name = ")
                .push_bind(&self.account_name)
                .push(", amount = ")
                .push_bind(self.amount)
                .push(", description = ")
                .push_bind(&self.description)
                .push(" WHERE id = ")
                .push_bind(self.id)
                .build()
                .execute(&mut *conn)
                .await?;
        }
        Self::get(db, self.id).await
    }

    /// Create transaction
    pub async fn create(&self, db: &Database) -> Result<Transaction> {
        let insert: Insert = {
            let mut conn = db.lock().await;
            let mut qry = QueryBuilder::<Sqlite>::new(
                r#"INSERT INTO transactions (
                    member_id,
                    date,
                    account_name,
                    amount,
                    description
                ) VALUES (
                "#,
            );
            qry.separated(", ")
                .push_bind(self.member_id)
                .push_bind(self.date)
                .push_bind(&self.account_name)
                .push_bind(self.amount)
                .push_bind(&self.description);

            qry.push(") RETURNING id ")
                .build_query_as()
                .fetch_one(&mut *conn)
                .await?
        };
        Self::get(db, insert.id).await
    }
}

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
