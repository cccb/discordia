use chrono::NaiveDate;

#[derive(Debug, Default, Clone)]
pub struct BankTransaction {
    pub id: u32,
    pub date: NaiveDate,
    pub name: String,
    pub iban: String,
    pub amount: f64,
    pub subject: String,
}
