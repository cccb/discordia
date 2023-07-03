use anyhow::Result;
use chrono::NaiveDate;
use sqlx::{FromRow, QueryBuilder, Sqlite};

use crate::{results::Insert, Connection};

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
    fn query<'q>(filter: &MemberFilter) -> QueryBuilder<'q, Sqlite> {
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

        if let Some(id) = filter.id {
            qry.push(" AND id = ").push_bind(id);
        }
        if let Some(name) = filter.name.clone() {
            qry.push(" AND name LIKE ").push_bind(format!("%{}%", name));
        }
        if let Some(email) = filter.email.clone() {
            qry.push(" AND email LIKE ").push_bind(email);
        }

        qry
    }

    /// Fetch members
    pub async fn filter(db: &Connection, filter: &MemberFilter) -> Result<Vec<Self>> {
        let mut conn = db.lock().await;
        let members: Vec<Self> = Self::query(filter)
            .build_query_as()
            .fetch_all(&mut *conn)
            .await?;
        Ok(members)
    }

    /// Fetch a single member by ID
    pub async fn get(db: &Connection, id: u32) -> Result<Self> {
        let mut conn = db.lock().await;
        let filter = MemberFilter {
            id: Some(id),
            ..MemberFilter::default()
        };
        let member: Self = Self::query(&filter)
            .build_query_as()
            .fetch_one(&mut *conn)
            .await?;
        Ok(member)
    }

    /// Update member
    pub async fn update(&self, db: &Connection) -> Result<Self> {
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
    pub async fn insert(&self, db: &Connection) -> Result<Self> {
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

    /// Delete member
    pub async fn delete(&self, db: &Connection) -> Result<()> {
        let mut conn = db.lock().await;
        QueryBuilder::<Sqlite>::new("DELETE FROM members WHERE id = ")
            .push_bind(self.id)
            .build()
            .execute(&mut *conn)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::connection;

    #[tokio::test]
    async fn test_member_insert() {
        let (_handle, conn) = connection::open_test().await;
        let today: NaiveDate = chrono::Local::now().date_naive();
        let member = Member {
            name: "Test Member".to_string(),
            email: "mail@test-member.eris".to_string(),
            membership_start: today,
            notes: "was very nice".to_string(),
            last_payment: NaiveDate::from_ymd_opt(1900, 1, 1).unwrap(),
            interval: 1,
            fee: 23.42,
            account: 42.32,
            ..Member::default()
        };
        let member = member.insert(&conn).await.unwrap();

        assert_eq!(member.name, "Test Member");
        assert_eq!(member.email, "mail@test-member.eris");
        assert_eq!(member.membership_start, today);
        assert_eq!(member.notes, "was very nice");
        assert_eq!(member.last_payment, NaiveDate::from_ymd_opt(1900, 1, 1).unwrap());
        assert_eq!(member.interval, 1);
        assert_eq!(member.fee, 23.42);
        assert_eq!(member.account, 42.32);
    }

    #[tokio::test]
    async fn test_member_update() {
        let (_handle, conn) = connection::open_test().await;
        let member = Member {
            name: "Test Member".to_string(),
            email: "eris@discordia.ccc".to_string(),
            ..Member::default()
        };
        let mut member = member.insert(&conn).await.unwrap();
        member.name = "Test Member Updated".to_string();
        member.email = "new@email".to_string();
        member.membership_start = NaiveDate::from_ymd_opt(1900, 2, 2).unwrap();
        member.membership_end = Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        member.last_payment = NaiveDate::from_ymd_opt(2023, 4, 2).unwrap();
        member.interval = 2;
        member.fee = 123.42;
        member.account = 23.0;
        member.notes = "was not very nice".to_string();

        let member = member.update(&conn).await.unwrap();
        assert_eq!(member.name, "Test Member Updated");
        assert_eq!(member.email, "new@email");
        assert_eq!(member.membership_start, NaiveDate::from_ymd_opt(1900, 2, 2).unwrap());
        assert_eq!(member.membership_end, Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()));
        assert_eq!(member.last_payment, NaiveDate::from_ymd_opt(2023, 4, 2).unwrap());
        assert_eq!(member.interval, 2);
        assert_eq!(member.fee, 123.42);
        assert_eq!(member.account, 23.0);
        assert_eq!(member.notes, "was not very nice");
    }

    #[tokio::test]
    async fn test_member_filter() {
        let (_handle, conn) = connection::open_test().await;
        // Insert two members
        let m1 = Member {
            name: "Test Member 1".to_string(),
            email: "test1@eris.discordia".to_string(),
            ..Member::default()
        };
        m1.insert(&conn).await.unwrap();

        let m2 = Member {
            name: "Test Member 2".to_string(),
            email: "test2@eris.discordia".to_string(),
            ..Member::default()
        };
        m2.insert(&conn).await.unwrap();

        // Filter by name
        let filter = MemberFilter {
            name: Some("Member 2".to_string()),
            ..MemberFilter::default()
        };

        let members = Member::filter(&conn, &filter).await.unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].name, "Test Member 2");
    }

    #[tokio::test]
    async fn test_member_delete() {
        let (_handle, conn) = connection::open_test().await;
        let m1 = Member {
            name: "Test Member 1".to_string(),
            email: "test1@eris.discordia".to_string(),
            ..Member::default()
        };
        m1.insert(&conn).await.unwrap();

        // Delete member again
        m1.delete(&conn).await.unwrap();
    }
}
