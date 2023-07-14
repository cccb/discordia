use chrono::{Datelike, Months, NaiveDate};
use anyhow::Result;
use thiserror::Error as ThisError;

use eris_data::{
    Retrieve,
    State,
    Member
};

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Member fee calculation start date {0}\
        is before last account calculation date {1}")]
    LastCalculationAfterStart(NaiveDate, NaiveDate),
    #[error("Member fee calculation end date {0}\
        is before last account calculation date {1}")]
    LastCalculationAfterEnd(NaiveDate, NaiveDate),
}

pub struct MemberFee {
    pub amount: f64,
    pub date: NaiveDate,
}

impl MemberFee {
    /// Get a description
    pub fn describe(&self) -> String {
        format!("Monthly member fee for {}", self.date.format("%B %Y"))
    }
}

/// Is member active?
/// Membership counts for the entire month. At least for
/// payments.
pub fn is_member_active(member: &Member, date: NaiveDate) -> bool {
    // Align dates to the first of the month
    let date = date.with_day(1).unwrap();
    let start = member.membership_start.with_day(1).unwrap();
    let end = member.membership_end.map(|d| d.with_day(1).unwrap());

    if date < start {
        return false;
    }
    if let Some(end) = end {
        if date > end {
            return false;
        }
    }

    true
}

/// Plausibility check: Is the member fee calculation start and 
/// end date after the `account_calculated_at` date stored in state
pub async fn check_member_fee_calculation_dates<DB>(
    db: &DB, 
    start: NaiveDate, 
    end: NaiveDate,
) -> Result<()>
where
    DB: Retrieve<State, Key=()>
{
    let state: State = db.retrieve(()).await?;
    // align dates
    let start = start.with_day(1).unwrap();
    let end = end.with_day(1).unwrap();

    // We assume calculation for the previous month.
    let calculated_at = state.accounts_calculated_at.with_day(1).unwrap();
    let calculated_at = calculated_at.checked_sub_months(Months::new(1)).unwrap();

    if start <= calculated_at {
        return Err(Error::LastCalculationAfterStart(
            start, 
            state.accounts_calculated_at,
        ).into());
    }
    if end <= calculated_at {
        return Err(Error::LastCalculationAfterEnd(
            end, 
            state.accounts_calculated_at,
        ).into());
    }
    Ok(())
}

pub trait CalculateFees {
    /// Caluculate member fees: Given is the monthly amount,
    /// the start date and the end date.
    fn calculate_fees(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Vec<MemberFee>;
}

impl CalculateFees for Member {
    /// Member fee calculation resulting in a list of member fees
    /// for a given date.
    fn calculate_fees(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Vec<MemberFee> {
        // Align dates to the first of the month, start with
        // beginning of membership. Test if member has payment
        // during calculation.
        let last_payment = self.last_payment.with_day(1).unwrap();
        let start = std::cmp::max(
            self.membership_start.with_day(1).unwrap(),
            start.with_day(1).unwrap(),
        );
        let end = end.with_day(1).unwrap();

        let mut fees = Vec::new();
        let mut date = start;

        while date <= end {
            if is_member_active(self, date) && date > last_payment {
                fees.push(MemberFee {
                    amount: self.fee,
                    date,
                });
            }
            // Advance one month, this is safe because we
            // aligned the dates to the first of the month.
            date = date.checked_add_months(Months::new(1)).unwrap();
        }
        fees
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eris_data::{Update};
    use eris_db::Connection;

    #[tokio::test]
    async fn test_check_member_fee_calculation_dates() {
        let db = Connection::open_test().await;
        let state = db.update(State {
            accounts_calculated_at: NaiveDate::from_ymd_opt(2023, 5, 6).unwrap(),
        }).await.unwrap();
        
        let start = NaiveDate::from_ymd_opt(2023, 5, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2023, 6, 1).unwrap();

        // This should be ok.
        check_member_fee_calculation_dates(&db, start, end).await.unwrap();
    }

    #[test]
    fn test_memberfee_describe() {
        let fee = MemberFee {
            amount: 23.0,
            date: NaiveDate::from_ymd_opt(2022, 3, 9).unwrap(),
        };
        assert_eq!(fee.describe(), "Monthly member fee for March 2022");
    }

    #[test]
    fn test_memberfee_calculation() {
        let mut member = Member {
            membership_start: NaiveDate::from_ymd_opt(2023, 5, 9).unwrap(),
            fee: 23.0,
            ..Default::default()
        };
        let fees = member.calculate_fees(
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2023, 7, 23).unwrap(),
        );
        assert_eq!(fees.len(), 3);

        // With a start date a month back
        let fees = member.calculate_fees(
            NaiveDate::from_ymd_opt(2023, 6, 1).unwrap(),
            NaiveDate::from_ymd_opt(2023, 7, 23).unwrap(),
        );
        assert_eq!(fees.len(), 2);

        // With a last payment in the last month
        member.last_payment = NaiveDate::from_ymd_opt(2023, 6, 9).unwrap();
        let fees = member.calculate_fees(
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2023, 7, 23).unwrap(),
        );
        assert_eq!(fees.len(), 1);
    }

    #[test]
    fn test_is_member_active() {
        let member = Member {
            membership_start: NaiveDate::from_ymd_opt(2022, 2, 23).unwrap(),
            ..Default::default()
        };
        assert_eq!(
            is_member_active(
                &member,
                NaiveDate::from_ymd_opt(2022, 1, 23).unwrap()
            ),
            false
        );
        assert_eq!(
            is_member_active(
                &member,
                NaiveDate::from_ymd_opt(2022, 2, 21).unwrap()
            ),
            true
        );
        assert_eq!(
            is_member_active(
                &member,
                NaiveDate::from_ymd_opt(2022, 4, 24).unwrap()
            ),
            true
        );

        let member = Member {
            membership_end: Some(NaiveDate::from_ymd_opt(2022, 2, 23).unwrap()),
            ..Default::default()
        };
        assert_eq!(
            is_member_active(
                &member,
                NaiveDate::from_ymd_opt(2022, 1, 22).unwrap()
            ),
            true
        );
        assert_eq!(
            is_member_active(
                &member,
                NaiveDate::from_ymd_opt(2022, 2, 25).unwrap()
            ),
            true
        );
        assert_eq!(
            is_member_active(
                &member,
                NaiveDate::from_ymd_opt(2022, 3, 1).unwrap()
            ),
            false
        );
    }
}
