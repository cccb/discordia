use chrono::{Datelike, Months, NaiveDate};

use eris_data::Member;

pub struct MemberFee {
    pub amount: f64,
    pub date: NaiveDate,
}

impl MemberFee {
    /// Get a description
    pub fn description(&self) -> String {
        format!("Monthly member fee for {}", self.date.format("%B %Y"))
    }
}

pub trait CalculateFees {
    /// Caluculate member fees: Given is the monthly amount,
    /// the start date and the end date.
    fn calculate_fees(&self, end: NaiveDate) -> Vec<MemberFee>;
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

impl CalculateFees for Member {
    /// Member fee calculation resulting in a list of member fees
    /// for a given date.
    fn calculate_fees(&self, end: NaiveDate) -> Vec<MemberFee> {
        // Align dates to the first of the month, start with
        // beginning of membership. Test if member has payment
        // during calculation.
        let last_payment = self.last_payment.with_day(1).unwrap();
        let start = self.membership_start.with_day(1).unwrap();
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

    #[test]
    fn test_memberfee_description() {
        let fee = MemberFee {
            amount: 23.0,
            date: NaiveDate::from_ymd_opt(2022, 3, 9).unwrap(),
        };
        assert_eq!(fee.description(), "Monthly member fee for March 2022");
    }

    #[test]
    fn test_memberfee_calculation() {
        let mut member = Member {
            membership_start: NaiveDate::from_ymd_opt(2023, 5, 9).unwrap(),
            fee: 23.0,
            ..Default::default()
        };
        let fees = member
            .calculate_fees(NaiveDate::from_ymd_opt(2023, 7, 23).unwrap());
        assert_eq!(fees.len(), 3);

        // With a last payment in the last month
        member.last_payment = NaiveDate::from_ymd_opt(2023, 6, 9).unwrap();
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