use std::fs::File;

use anyhow::Result;
use csv::{ReaderBuilder, StringRecord};
use encoding_rs::WINDOWS_1252;

use encoding_rs_io::DecodeReaderBytesBuilder;

use crate::{deuba::Language, BankTransaction};

impl BankTransaction {
    pub fn from_record(
        num: u32,
        lang: &Language,
        record: &StringRecord,
    ) -> Result<Option<Self>> {
        if record.len() < 18 {
            return Ok(None);
        }
        // Fields:
        //  0: booking date
        //  1: value date
        //  2: type (ignored)
        //  3: name
        //  4: subject
        //  5: IBAN
        //  6: BIC (ignored)
        //  ...
        //  15: outgoing amount
        //  16: incoming amount
        //  17: currency

        // A valid record starts with two dates
        let value_date = lang.parse_date(&record[1]);
        if value_date.is_err() {
            return Ok(None);
        }
        let booking_date = lang.parse_date(&record[0])?;
        let name = &record[3];
        let subject = &record[4];
        let iban = &record[5];

        if &record[16] == "" {
            return Ok(None);
        }

        let amount = lang.parse_number(&record[16])?;

        Ok(Some(Self {
            id: num,
            date: booking_date,
            name: name.to_string(),
            iban: iban.to_string(),
            subject: subject.to_string(),
            amount: amount,
        }))
    }
}

/// Parse a Deutsche Bank CSV export.
/// Only incoming transactions are considered.
pub fn parse(file: &mut File) -> Result<Vec<BankTransaction>> {
    let lang = Language::from_file(file)?;
    let transcoder = DecodeReaderBytesBuilder::new()
        .encoding(Some(WINDOWS_1252))
        .build(file);

    let mut rdr = ReaderBuilder::new()
        .flexible(true)
        .delimiter(b';')
        .from_reader(transcoder);
    let mut transactions: Vec<BankTransaction> = vec![];
    let mut counter = 0;

    // I'm sure there is a more elegant way to do this
    for result in rdr.records() {
        counter += 1;
        let tx = BankTransaction::from_record(counter, &lang, &result?)?;
        if let Some(tx) = tx {
            transactions.push(tx);
        }
    }
    Ok(transactions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_de() {
        let mut file = File::open("test/konto_de.csv").unwrap();
        let txs = parse(&mut file).unwrap();
        assert_eq!(txs.len(), 2);
    }

    #[test]
    fn test_parse_en() {
        let mut file = File::open("test/konto_en.csv").unwrap();
        let txs = parse(&mut file).unwrap();
        assert_eq!(txs.len(), 5);
    }
}
