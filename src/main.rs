use anyhow::Result;
use chrono::{Local, NaiveDate};

use discordia::{
    cli, database,
    models::{BankImportRule, Member, MemberFilter, State},
};

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::parse();
    println!("using: {:?}", args);

    let db = database::connect(&args.db).await?;
    let today = Local::now().date_naive();

    let m = Member {
        name: "NewMember Test".into(),
        email: "test@foo.bar".into(),
        membership_start: today,
        ..Member::default()
    };

    /*
    let m = m.insert(&db).await?;

    println!("{:?}", m);
    */
    let rules = BankImportRule::filter(&db, None).await?;
    println!("{:?}", rules);

    /*
    let filter = MemberFilter {
    id: Some(106),
    ..MemberFilter::default()
    };
    let members = Member::filter(&db, Some(filter)).await?;

    for m in members.iter() {
    println!("member: {:?}", m);
    }

    let mut m = Member::get(&db, 106).await?;
    m.name = "Anni".into();
    m.account = 2342.42;
    let m = m.update(&db).await?;

    println!("member: {:?}", m);
                 */

    Ok(())
}
