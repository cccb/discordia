use anyhow::Result;
use async_trait::async_trait;
use sqlx::{QueryBuilder, Sqlite};

use eris_data::{
    Delete,
    Update,
    Insert,
    Query,
    Retrieve,
    Member,
    MemberFilter,
};

use crate::{
    results::{Id, QueryError},
    Connection,
};

#[async_trait]
impl Query<Member> for Connection {
    type Filter = MemberFilter;
    async fn query(&self, filter: &Self::Filter) -> Result<Vec<Member>> {
        let mut conn = self.lock().await;
        let mut qry = QueryBuilder::new(
            r#"
            SELECT 
                id,
                name,
                email,
                notes,
                membership_start,
                membership_end,
                last_payment_at,
                account_calculated_at,
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

        let members: Vec<Member> = qry.build_query_as().fetch_all(&mut *conn).await?;
        Ok(members)
    }
}

#[async_trait]
impl Retrieve<Member> for Connection {
    type Key = u32;
    async fn retrieve(&self, member_id: Self::Key) -> Result<Member> {
        let filter = MemberFilter {
            id: Some(member_id),
            ..Default::default()
        };
        let member = self
            .query(&filter)
            .await?
            .pop()
            .ok_or_else(|| QueryError::NotFound)?;
        Ok(member)
    }
}

#[async_trait]
impl Insert<Member> for Connection {
    async fn insert(&self, member: Member) -> Result<Member> {
        let insert: Id<u32> = {
            let mut conn = self.lock().await;
            let mut qry = QueryBuilder::<Sqlite>::new(
                r#"INSERT INTO members (
                    name,
                    email,
                    notes,
                    membership_start,
                    membership_end,
                    last_payment_at,
                    account_calculated_at,
                    interval,
                    fee,
                    account
                ) VALUES (
                "#,
            );
            qry.separated(", ")
                .push_bind(&member.name)
                .push_bind(&member.email)
                .push_bind(&member.notes)
                .push_bind(member.membership_start)
                .push_bind(member.membership_end)
                .push_bind(member.last_payment_at)
                .push_bind(member.account_calculated_at)
                .push_bind(member.interval)
                .push_bind(format!("{}", member.fee))
                .push_bind(format!("{}", member.account));

            qry.push(") RETURNING id ")
                .build_query_as()
                .fetch_one(&mut *conn)
                .await?
        };
        self.retrieve(insert.id).await
    }
}


#[async_trait]
impl Update<Member> for Connection {
    /// Update member
    async fn update(&self, member: Member) -> Result<Member> {
        {
            let mut conn = self.lock().await;
            QueryBuilder::<Sqlite>::new("UPDATE members SET")
                .push(" name = ")
                .push_bind(&member.name)
                .push(", email = ")
                .push_bind(&member.email)
                .push(", notes = ")
                .push_bind(&member.notes)
                .push(", membership_start = ")
                .push_bind(member.membership_start)
                .push(", membership_end = ")
                .push_bind(member.membership_end)
                .push(", last_payment_at = ")
                .push_bind(member.last_payment_at)
                .push(", account_calculated_at = ")
                .push_bind(member.account_calculated_at)
                .push(", interval = ")
                .push_bind(member.interval)
                .push(", fee = ")
                .push_bind(format!("{}", member.fee))
                .push(", account = ")
                .push_bind(format!("{}", member.account))
                .push(" WHERE id = ")
                .push_bind(member.id)
                .build()
                .execute(&mut *conn)
                .await?;
        }
        self.retrieve(member.id).await  
    }
}

#[async_trait]
impl Delete<Member> for Connection {
    /// Delete member
    async fn delete(&self, member: Member) -> Result<()> {
        let mut conn = self.lock().await;
        QueryBuilder::<Sqlite>::new("DELETE FROM members WHERE id = ")
            .push_bind(member.id)
            .build()
            .execute(&mut *conn)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    use eris_data::Transaction;

    #[tokio::test]
    async fn test_member_insert() {
        let db = Connection::open_test().await;
        let today: NaiveDate = chrono::Local::now().date_naive();
        let member = Member {
            name: "Test Member".to_string(),
            email: "mail@test-member.eris".to_string(),
            membership_start: today,
            notes: "was very nice".to_string(),
            last_payment_at: NaiveDate::from_ymd_opt(1900, 1, 1).unwrap(),
            interval: 1,
            fee: 23.42,
            account: 42.32,
            ..Member::default()
        };
        let member = db.insert(member).await.unwrap();

        assert_eq!(member.name, "Test Member");
        assert_eq!(member.email, "mail@test-member.eris");
        assert_eq!(member.membership_start, today);
        assert_eq!(member.notes, "was very nice");
        assert_eq!(member.last_payment_at, NaiveDate::from_ymd_opt(1900, 1, 1).unwrap());
        assert_eq!(member.interval, 1);
        assert_eq!(member.fee, 23.42);
        assert_eq!(member.account, 42.32);
    }

    #[tokio::test]
    async fn test_member_update() {
        let db = Connection::open_test().await;
        let member = Member {
            name: "Test Member".to_string(),
            email: "eris@discordia.ccc".to_string(),
            ..Member::default()
        };
        let mut member = db.insert(member).await.unwrap();
        member.name = "Test Member Updated".to_string();
        member.email = "new@email".to_string();
        member.membership_start = NaiveDate::from_ymd_opt(1900, 2, 2).unwrap();
        member.membership_end = Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        member.last_payment_at = NaiveDate::from_ymd_opt(2023, 4, 2).unwrap();
        member.interval = 2;
        member.fee = 123.42;
        member.account = 23.0;
        member.notes = "was not very nice".to_string();

        let member = db.update(member).await.unwrap();
        assert_eq!(member.name, "Test Member Updated");
        assert_eq!(member.email, "new@email");
        assert_eq!(member.membership_start, NaiveDate::from_ymd_opt(1900, 2, 2).unwrap());
        assert_eq!(member.membership_end, Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()));
        assert_eq!(member.last_payment_at, NaiveDate::from_ymd_opt(2023, 4, 2).unwrap());
        assert_eq!(member.interval, 2);
        assert_eq!(member.fee, 123.42);
        assert_eq!(member.account, 23.0);
        assert_eq!(member.notes, "was not very nice");
    }

    #[tokio::test]
    async fn test_member_filter() {
        let db = Connection::open_test().await;
        // Insert two members
        let m1 = Member {
            name: "Test Member 1".to_string(),
            email: "test1@eris.discordia".to_string(),
            ..Member::default()
        };
        db.insert(m1).await.unwrap();

        let m2 = Member {
            name: "Test Member 2".to_string(),
            email: "test2@eris.discordia".to_string(),
            ..Member::default()
        };
        db.insert(m2).await.unwrap();

        // Filter by name
        let filter = MemberFilter {
            name: Some("Member 2".to_string()),
            ..MemberFilter::default()
        };
        let members: Vec<Member> = db.query(&filter).await.unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].name, "Test Member 2");
    }

    #[tokio::test]
    async fn test_member_query_name_like() {
        let db = Connection::open_test().await;
        db.insert(Member {
            name: "Test Member".to_string(),
            ..Default::default()
        }).await.unwrap();

        let result = MemberFilter {
            name: Some("tEsT MeMber".to_string()),
            ..MemberFilter::default()
        };
        let members: Vec<Member> = db.query(&result).await.unwrap();
        assert_eq!(members.len(), 1);

        let result = MemberFilter {
            name: Some("f3st MeMber".to_string()),
            ..MemberFilter::default()
        };
        let members: Vec<Member> = db.query(&result).await.unwrap();
        assert_eq!(members.len(), 0);
    }

    #[tokio::test]
    async fn test_member_delete() {
        let db = Connection::open_test().await;
        let member = Member {
            name: "Test Member 1".to_string(),
            email: "test1@eris.discordia".to_string(),
            ..Member::default()
        };
        let member = db.insert(member).await.unwrap();

        // Delete member again
        db.delete(member).await.unwrap();
    }

    #[tokio::test]
    async fn test_member_get_related_transactions() {
        let db = Connection::open_test().await;

        // Create test member
        let m = Member{
            name: "Testmember".to_string(),
            ..Default::default()
        };
        let m = db.insert(m).await.unwrap();

        // Create transaction for member
        let tx = Transaction {
            member_id: m.id,
            ..Default::default()
        };
        db.insert(tx).await.unwrap();
        let tx = Transaction {
            member_id: m.id,
            ..Default::default()
        };
        db.insert(tx).await.unwrap();

        // Get related transactions
        let txs = m.get_transactions(&db).await.unwrap();
        assert_eq!(txs.len(), 2);
    }
}
