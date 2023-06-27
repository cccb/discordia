
use chrono::{Datelike, NaiveDate};


pub fn last_month() -> NaiveDate {
    let now = chrono::Local::now().date_naive();
    now
        .with_day(1)
        .unwrap()
        .pred_opt()
        .unwrap()
        .with_day(1)
        .unwrap()
}


#[cfg(test)]
mod tests {
    use super::*;

    use chrono::Duration;

    #[test]
    fn test_last_month() {
        let today = chrono::Local::now().date_naive();
        let date = last_month();
        println!("last: {:?}", date);
    }
}


