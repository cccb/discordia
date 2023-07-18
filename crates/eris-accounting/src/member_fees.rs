use chrono::{Months, NaiveDate};
use thiserror::Error as ThisError;

use eris_data::Member;

use crate::datetime::AlignStart;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error(
        "Member fee calculation start date {0}\
        is before last account calculation date {1}"
    )]
    LastCalculationAfterStart(NaiveDate, NaiveDate),
    #[error(
        "Member fee calculation end date {0}\
        is before last account calculation date {1}"
    )]
    LastCalculationAfterEnd(NaiveDate, NaiveDate),
}

/// A monthly membership fee.
pub struct MemberFee {
    pub amount: f64,
    pub date: NaiveDate,
}

impl MemberFee {
    /// Get a description for the membership fee transaction.
    pub fn describe(&self) -> String {
        format!("Monthly member fee for {}", self.date.format("%B %Y"))
    }
}

/// Is member active?
/// Membership counts for the entire month. At least for
/// payments.
pub fn is_member_active(member: &Member, date: NaiveDate) -> bool {
    // Align dates to the first of the month
    let date = date.align_start();
    let start = member.membership_start.align_start();
    let end = member.membership_end.map(|d| d.align_start());

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

pub trait CalculateFees {
    /// Caluculate member fees: Given is the monthly amount,
    /// the start date and the end date.
    fn calculate_fees(&self, end: NaiveDate) -> Vec<MemberFee>;
}

impl CalculateFees for Member {
    /// Member fee calculation resulting in a list of member fees
    /// for a given date.
    fn calculate_fees(&self, end: NaiveDate) -> Vec<MemberFee> {
        // Align dates to the first of the month, start with
        // beginning of membership. Test if member has payment
        // during calculation.
        let last_calculation = self.account_calculated_at.align_start();
        let last_payment = self.last_payment_at.align_start();
        let start =
            last_calculation.checked_add_months(Months::new(1)).unwrap();
        let start = std::cmp::max(self.membership_start.align_start(), start);
        let end = end.align_start();
        if start > end {
            return vec![];
        }

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
            membership_start: NaiveDate::from_ymd_opt(2023, 4, 9).unwrap(),
            fee: 23.0,
            ..Default::default()
        };
        let fees = member
            .calculate_fees(NaiveDate::from_ymd_opt(2023, 7, 23).unwrap());
        assert_eq!(fees.len(), 4);
        member.account_calculated_at =
            NaiveDate::from_ymd_opt(2023, 4, 9).unwrap();

        let fees = member
            .calculate_fees(NaiveDate::from_ymd_opt(2023, 7, 23).unwrap());
        assert_eq!(fees.len(), 3);

        // With a last payment in the last month
        member.last_payment_at = NaiveDate::from_ymd_opt(2023, 6, 9).unwrap();
        let fees = member
            .calculate_fees(NaiveDate::from_ymd_opt(2023, 7, 23).unwrap());
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
