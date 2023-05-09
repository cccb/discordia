use anyhow::Result;
use chrono::NaiveDate;
use sqlx::{FromRow, QueryBuilder, Sqlite};

use crate::{database::Database, db::results::Insert};

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
    pub async fn filter(db: &Database, filter: Option<MemberFilter>) -> Result<Vec<Self>> {
        let mut conn = db.lock().await;
        let members: Vec<Self> = Self::query(filter)
            .build_query_as()
            .fetch_all(&mut *conn)
            .await?;
        Ok(members)
    }

    /// Fetch a single member by ID
    pub async fn get(db: &Database, id: u32) -> Result<Self> {
        let mut conn = db.lock().await;
        let filter = MemberFilter {
            id: Some(id),
            ..MemberFilter::default()
        };
        let member: Self = Self::query(Some(filter))
            .build_query_as()
            .fetch_one(&mut *conn)
            .await?;
        Ok(member)
    }

    /// Update member
    pub async fn update(&self, db: &Database) -> Result<Self> {
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
    pub async fn insert(&self, db: &Database) -> Result<Self> {
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
