use anyhow::Result;
use async_trait::async_trait;
use sqlx::{QueryBuilder, Sqlite};

use eris_data::{
    BankImportRule,
    BankImportRuleFilter,
    Retrieve,
    Query,
    Update,
    Delete,
    Insert,
};

use crate::{
    Connection,
    QueryError,
};


#[async_trait]
impl Query<BankImportRule> for Connection {
    type Filter = BankImportRuleFilter;

    /// Fetch member IBANs
    async fn query(
        &self, filter: &BankImportRuleFilter,
    ) -> Result<Vec<BankImportRule>> {
        let mut conn = self.lock().await;
        let mut qry = QueryBuilder::new(
            r#"
            SELECT 
                member_id,
                iban,
                match_subject,
                ROUND(split_amount, 10) AS split_amount
            FROM bank_import_member_ibans
            WHERE 1
            "#,
        );
        if let Some(id) = filter.member_id {
            qry.push(" AND member_id = ").push_bind(id);
        }
        if let Some(iban) = filter.iban.clone() {
            qry.push(" AND iban = ").push_bind(iban);
        }
        let rules: Vec<BankImportRule> = qry.build_query_as()
            .fetch_all(&mut *conn)
            .await?;
        Ok(rules)
    }
}

#[async_trait]
impl Retrieve<BankImportRule> for Connection {
    type Key = (u32, String);

    // Get a single member IBAN rule
    async fn retrieve(
        &self,
        (member_id, iban): Self::Key,
    ) -> Result<BankImportRule> {
        let filter = BankImportRuleFilter{
            member_id: Some(member_id),
            iban: Some(iban),
        };
        let rules: Vec<BankImportRule> = self.query(&filter).await?;
        if rules.len() == 0 {
            return Err(QueryError::NotFound.into());
        }
        if rules.len() > 1 {
            return Err(QueryError::Ambiguous(rules.len()).into());
        }
        Ok(rules[0].clone())
    }
}

#[async_trait]
impl Update<BankImportRule> for Connection {
    /// Update member IBAN
    async fn update(self, rule: BankImportRule) -> Result<BankImportRule> {
        {
            let mut conn = self.lock().await;
            let mut split_amount: Option<String> = None;
            if let Some(amount) = rule.split_amount {
                split_amount = Some(format!("{}", amount)); 
            }

            QueryBuilder::<Sqlite>::new(
                "UPDATE bank_import_member_ibans SET")
                .push(" split_amount = ")
                .push_bind(&split_amount)
                .push(", match_subject = ")
                .push_bind(&rule.match_subject)
                .push(" WHERE member_id = ")
                .push_bind(rule.member_id)
                .push(" AND iban = ")
                .push_bind(&rule.iban)
                .build()
                .execute(&mut *conn)
                .await?;
        }
        self.retrieve((rule.member_id, rule.iban.clone())).await
    }

}

#[async_trait]
impl Insert<BankImportRule> for Connection {
    /// Create member IBAN rule
    async fn insert(
        &self,
        rule: BankImportRule,
    ) -> Result<BankImportRule> {
        {
            let mut split_amount: Option<String> = None;
            if let Some(amount) = rule.split_amount {
                split_amount = Some(format!("{}", amount)); 
            }

            let mut conn = self.lock().await;
            let mut qry = QueryBuilder::<Sqlite>::new(
                r#"INSERT INTO bank_import_member_ibans (
                    member_id,
                    iban,
                    match_subject,
                    split_amount
            "#,
            );
            qry.push(" ) VALUES ( ");
            qry.separated(", ")
                .push_bind(rule.member_id)
                .push_bind(&rule.iban)
                .push_bind(&rule.match_subject)
                .push_bind(&split_amount);
            qry.push(") ");
            qry.build()
                .execute(&mut *conn).await?;
        }
        self.retrieve((rule.member_id, rule.iban.clone())).await
    }

}

#[async_trait]
impl Delete<BankImportRule> for Connection {

    /// Delete a member IBAN rule
    async fn delete(&self, rule: BankImportRule) -> Result<()> {
        let mut conn = self.lock().await;
        QueryBuilder::<Sqlite>::new(
            "DELETE FROM bank_import_member_ibans WHERE")
            .push(" member_id = ")
            .push_bind(rule.member_id)
            .push(" AND iban = ")
            .push_bind(&rule.iban)
            .build()
            .execute(&mut *conn)
            .await?;

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use eris_data::Member;

    #[tokio::test]
    async fn test_bank_import_member_iban_insert() {
        let db = Connection::open_test().await;

        let m = Member{
            name: "Testmember1".to_string(),
            ..Member::default()
        };
        let m = db.insert(m).await.unwrap();

        let rule = BankImportRule{
            member_id: m.id,
            iban: "DE2342123456".to_string(),
            split_amount: None,
            match_subject: Some("beitrag".to_string()),
        };
        let rule = db.insert(rule).await.unwrap();
        assert_eq!(rule.member_id, m.id);
        assert_eq!(rule.iban, "DE2342123456");
        assert_eq!(rule.match_subject, Some("beitrag".to_string()));
    }

    #[tokio::test]
    async fn test_bank_import_member_iban_update() {
        let db = Connection::open_test().await;

        let m = Member{
            name: "Testmember1".to_string(),
            ..Member::default()
        };
        let m = db.insert(m).await.unwrap();

        let rule = BankImportRule{
            member_id: m.id,
            iban: "DE2342123456".to_string(),
            split_amount: Some(23.42),
            match_subject: None,
        };
        let mut rule = db.insert(rule).await.unwrap();

        assert_eq!(rule.match_subject, None);
        assert_eq!(rule.split_amount, Some(23.42));

        // Update rule
        rule.match_subject = Some("beitrag".to_string());
        rule.split_amount = None;

        let rule = db.update(rule).await.unwrap();

        assert_eq!(rule.member_id, m.id);
        assert_eq!(rule.match_subject, Some("beitrag".to_string()));
        assert_eq!(rule.split_amount, None);
    }

    #[tokio::test]
    async fn test_bank_import_member_iban_delete() {
        let conn = Connection::open_test().await;
        let m = Member{
            name: "Testmember1".to_string(),
            ..Member::default()
        };
        let m = conn.insert(m).await.unwrap();

        let rule = BankImportRule{
            member_id: m.id,
            iban: "foo".to_string(),
            split_amount: Some(23.42),
            match_subject: None,
        };
        let rule = conn.insert(rule).await.unwrap();

        conn.delete(rule).await.unwrap();
    }
}
