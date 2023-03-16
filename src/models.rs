use anyhow::Result;
use chrono::NaiveDate;
use serde_json::Value as JSONValue;
use sqlx::{FromRow, QueryBuilder, Sqlite};

use crate::database::Database;

#[derive(Debug, Clone, FromRow)]
pub struct Insert {
    pub id: u32,
}

#[derive(Debug, Clone, FromRow)]
pub struct InsertString {
    pub id: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct State {
    pub accounts_calculated_at: NaiveDate,
}

impl State {
    /// Fetch current state from database
    pub async fn fetch(db: &Database) -> Result<Self> {
        let mut conn = db.lock().await;
        let state: State = sqlx::query_as("SELECT accounts_calculated_at FROM state")
            .fetch_one(&mut *conn)
            .await?;
        Ok(state)
    }

    pub async fn update(&self, db: &Database) -> Result<Self> {
        {
            let mut conn = db.lock().await;
            QueryBuilder::new("UPDATE state SET")
                .push(" accounts_calculated_at = ")
                .push_bind(self.accounts_calculated_at)
                .build()
                .execute(&mut *conn)
                .await?;
        }
        Self::fetch(db).await
    }
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
    pub async fn filter(db: &Database, filter: Option<MemberFilter>) -> Result<Vec<Member>> {
        let mut conn = db.lock().await;
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

#[derive(Default, Clone, Debug, FromRow)]
pub struct Transaction {
    pub id: u32,
    pub member_id: u32,
}

#[derive(Default, Clone, Debug, FromRow)]
pub struct BankImportRule {
    pub iban_hash: String,
    pub member_id: u32,
    pub handler: String,
    pub params: Option<JSONValue>, // JSON
}

pub type BankImportParamSplit = Vec<BankImportSplit>;

pub struct BankImportSplit {
    pub member_id: u32,
    pub amount: f64,
}

#[derive(Default, Clone, Debug)]
pub struct BankImportRuleFilter {
    pub iban_hash: Option<String>,
    pub member_id: Option<u32>,
}

impl BankImportRule {
    /// Make select query
    fn query<'q>(filter: Option<BankImportRuleFilter>) -> QueryBuilder<'q, Sqlite> {
        let mut qry = QueryBuilder::new(
            r#"
            SELECT
              iban_hash,
              member_id,
              handler,
              params
            FROM bank_import_rules
            WHERE 1
        "#,
        );
        if let Some(filter) = filter {
            if let Some(iban_hash) = filter.iban_hash {
                qry.push(" iban_hash = ").push_bind(iban_hash);
            }
            if let Some(member_id) = filter.member_id {
                qry.push(" member_id = ").push_bind(member_id);
            }
        }
        qry
    }

    pub async fn filter(
        db: &Database,
        filter: Option<BankImportRuleFilter>,
    ) -> Result<Vec<BankImportRule>> {
        let mut conn = db.lock().await;
        let rules = Self::query(filter)
            .build_query_as()
            .fetch_all(&mut *conn)
            .await?;
        Ok(rules)
    }

    pub async fn get(db: &Database, iban_hash: &str) -> Result<BankImportRule> {
        let mut conn = db.lock().await;
        let filter = BankImportRuleFilter {
            iban_hash: Some(iban_hash.into()),
            ..BankImportRuleFilter::default()
        };
        let rule: BankImportRule = Self::query(Some(filter))
            .build_query_as()
            .fetch_one(&mut *conn)
            .await?;
        Ok(rule)
    }

    pub async fn insert(db: &Database, rule: &BankImportRule) -> Result<BankImportRule> {
        let insert: InsertString = {
            let mut conn = db.lock().await;
            let mut qry = QueryBuilder::new(
                r#"
                INSERT INTO bank_import_rules (
                    iban_hash,
                    member_id,
                    handler,
                    params
                ) VALUES ( "#,
            );

            qry.separated(", ")
                .push_bind(&rule.iban_hash)
                .push_bind(rule.member_id);

            qry.push(") RETURNING iban_hash AS id")
                .build_query_as()
                .fetch_one(&mut *conn)
                .await?
        };
        Self::get(db, &insert.id).await
    }
}
