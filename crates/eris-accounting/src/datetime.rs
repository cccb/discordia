use chrono::{Datelike, NaiveDate};

/// Get current date
pub fn today() -> NaiveDate {
    chrono::Local::now().date_naive()
}

/// Get last month relative to now, aligned
/// to the beginning of the month.
pub fn last_month() -> NaiveDate {
    today()
        .with_day(1)
        .unwrap()
        .pred_opt()
        .unwrap()
        .with_day(1)
        .unwrap()
}

/// Get the number of months between two dates.
/// This only accounts for full months. The days
/// are irrelevant.
pub trait CountMonths {
    fn count_months(&self, other: &Self) -> u64;
}

/// Implement difference in months for two dates.
impl CountMonths for NaiveDate {
    fn count_months(&self, other: &Self) -> u64 {
        let years = (other.year() - self.year()).abs() as i64;
        let months = (other.month() as i32 - self.month() as i32) as i64;
        (years * 12 + months) as u64
    }
}

/// It is inconvenient to always write with_day 1 unwrap,
/// so we add align_start() to NaiveDate.
pub trait AlignStart {
    fn align_start(self) -> Self;
}

impl AlignStart for NaiveDate {
    fn align_start(self) -> Self {
        self.with_day(1).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_last_month() {
        let date = last_month();
        println!("last: {:?}", date);
    }

    #[test]
    fn test_count_months() {
        let d1 = NaiveDate::from_ymd_opt(2022, 11, 15).unwrap();
        let d2 = NaiveDate::from_ymd_opt(2022, 12, 20).unwrap();
        let d3 = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let d4 = NaiveDate::from_ymd_opt(2023, 2, 2).unwrap();

        assert_eq!(d1.count_months(&d2), 1);
        assert_eq!(d1.count_months(&d3), 2);
        assert_eq!(d1.count_months(&d4), 3);
    }
}
