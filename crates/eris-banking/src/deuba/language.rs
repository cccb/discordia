use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

use anyhow::Result;
use chrono::NaiveDate;

#[derive(Debug, PartialEq)]
pub enum Language {
    DE,
    EN,
}

impl Language {
    /// Get the language used in the transactions CSV file
    pub fn from_file(file: &mut File) -> Result<Self> {
        // Read first couple of bytes to determine the language
        let mut buf = vec![0; 80];
        file.seek(SeekFrom::Start(0))?;
        file.read_exact(&mut buf)?;
        file.seek(SeekFrom::Start(0))?;

        // Check if the buf includes 'Transactions'
        let content = String::from_utf8_lossy(&buf);
        let lang = if content.contains("Transactions") {
            Language::EN
        } else {
            Language::DE
        };
        Ok(lang)
    }

    // Decode language dependent date format
    pub fn parse_date(&self, date: &str) -> Result<NaiveDate> {
        let date = match self {
            Language::DE => {
                // Date format: DD.MM.YYYY
                NaiveDate::parse_from_str(date, "%d.%m.%Y")?
            }
            Language::EN => {
                // Date format (cursed): MM/DD/YYYY
                NaiveDate::parse_from_str(date, "%m/%d/%Y")?
            }
        };
        Ok(date)
    }

    // Decode language dependent number
    pub fn parse_number(&self, number: &str) -> Result<f64> {
        let number = match self {
            Language::DE => {
                // Number format: 1.234,56
                number.replace(".", "").replace(",", ".").parse::<f64>()?
            }
            Language::EN => {
                // Number format: 1,234.56
                number.replace(",", "").parse::<f64>()?
            }
        };
        Ok(number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_file() {
        let mut file = File::open("test/konto_de.csv").unwrap();
        let lang = Language::from_file(&mut file).unwrap();
        assert_eq!(lang, Language::DE);

        let mut file = File::open("test/konto_en.csv").unwrap();
        let lang = Language::from_file(&mut file).unwrap();
        assert_eq!(lang, Language::EN);
    }

    #[test]
    fn test_parse_date_de() {
        let date = Language::DE.parse_date("09.12.1999").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(1999, 12, 9).unwrap());
    }

    #[test]
    fn test_parse_date_en() {
        let date = Language::EN.parse_date("12/09/1999").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(1999, 12, 9).unwrap());
    }
}
