
use anyhow::{anyhow, Result};
use chrono::NaiveDate;
use clap::{Subcommand, Args};
use inquire::Confirm;

use eris_data::{Member, MemberFilter, Query, Insert, Retrieve, Delete, Update, Transaction};
use eris_accounting::{datetime};
use eris_db::Connection;

use crate::formatting::PrintFormatted;

#[derive(Subcommand, Debug)]
pub enum Members {
    /// Show a member
    #[clap(name="show")]
    Show(ShowMember),
    /// List members
    #[clap(name="list")]
    List(ListMembers),
    /// Add a member
    #[clap(name="add")]
    Add(AddMember),
    /// Update a member
    #[clap(name="set")]
    Update(UpdateMember),
    /// Delete a member
    #[clap(name="delete")]
    Delete(DeleteMember),
}

impl Members {
    pub async fn run(self, db: &Connection) -> Result<()> {
        match self {
            Members::Show(cmd) => cmd.run(db).await,
            Members::List(cmd) => cmd.run(db).await,
            Members::Add(cmd) => cmd.run(db).await,
            Members::Update(cmd) => cmd.run(db).await,
            Members::Delete(cmd) => cmd.run(db).await,
        } 
    }
}

#[derive(Args, Debug)]
pub struct ShowMember {
    #[clap(short, long)]
    pub id: u32,
}

impl ShowMember {
    /// Run the command and show a member
    pub async fn run(self, db: &Connection) -> Result<()> {
        let member: Member = db.retrieve(self.id).await?;
        println!("");
        member.print_formatted();
        println!("");
        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct ListMembers {
    #[clap(short, long)]
    pub id: Option<u32>,
    #[clap(short, long)]
    pub name: Option<String>,
    #[clap(short, long)]
    pub email: Option<String>,
}

impl ListMembers {
    /// Run the command and list members
    pub async fn run(self, db: &Connection) -> Result<()> {
        // Create member filter
        let filter = MemberFilter{
            id: self.id,
            name: self.name,
            email: self.email,
            ..Default::default()
        };

        let members: Vec<Member> = db.query(&filter).await?;
        println!("{} members.", members.len());
        members.print_formatted();

        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct AddMember{
    #[clap(short, long)]
    pub name: String,
    #[clap(short, long)]
    pub email: String,
    #[clap(short='c', long)]
    pub notes: Option<String>,
    #[clap(long)]
    pub membership_start: Option<NaiveDate>,
    #[clap(short, long, default_value_t=20.0)]
    pub fee: f64,
    #[clap(short='p', long, default_value_t=1)]
    pub interval: u8,
    #[clap(short, long)]
    pub account: Option<f64>,
}

impl AddMember {
    /// Run the command and add a member to the database
    pub async fn run(self, db: &Connection) -> Result<()>
    {
        let membership_start = self.membership_start.unwrap_or(datetime::today());

        // Check if a member with this email already exists
        let members: Vec<Member> = db.query(&MemberFilter{
            email: Some(self.email.clone()),
            ..Default::default()
        }).await?;
        if members.len() > 0 {
            return Err(anyhow!(
                "Member with email {} already exists.", self.email));
        }

        let account = self.account.unwrap_or(-self.fee);

        let member = Member{
            name: self.name,
            email: self.email,
            notes: self.notes.unwrap_or("".to_string()),
            membership_start: membership_start,
            fee: self.fee,
            interval: self.interval,
            account: account,
            ..Default::default()
        };

        println!("");
        member.print_formatted();
        println!("");

        // Confirm adding member
        let confirm = Confirm::new("Add member?").with_default(true);
        if !confirm.prompt()? {
            return Ok(());
        }

        let member = db.insert(member).await?;
        println!("Member added with id {}.", member.id);

        Ok(())
    }
}


#[derive(Args, Debug)]
pub struct UpdateMember{
    #[clap(short, long)]
    pub id: u32,
    #[clap(short, long)]
    pub name: Option<String>,
    #[clap(short, long)]
    pub email: Option<String>,
    #[clap(short='c', long)]
    pub notes: Option<String>,
    #[clap(long)]
    pub membership_start: Option<NaiveDate>,
    #[clap(long)]
    pub membership_end: Option<NaiveDate>,
    #[clap(short, long)]
    pub fee: Option<f64>,
    #[clap(short='p', long)]
    pub interval: Option<u8>,
    #[clap(short, long)]
    pub account: Option<f64>,
}

impl UpdateMember {
    /// Run command and update a member
    pub async fn run(self, db: &Connection) -> Result<()> {
        let member: Member = db.retrieve(self.id).await?;
        let mut update = member.clone();
    
        if let Some(name) = self.name {
            update.name = name.clone();
        }
        if let Some(email) = self.email {
            update.email = email.clone();
        }
        if let Some(notes) = self.notes {
            update.notes = notes.clone();
        }
        if let Some(membership_start) = self.membership_start {
            update.membership_start = membership_start;
        }
        if let Some(membership_end) = self.membership_end {
            update.membership_end = Some(membership_end);
        }
        if let Some(fee) = &self.fee {
            update.fee = *fee;
        }
        if let Some(interval) = self.interval {
            update.interval = interval;
        }
        if let Some(account) = self.account {
            update.account = account;
        }

        println!("");
        (member.clone(), update.clone()).print_formatted();
        println!("");
        let confirm = Confirm::new("Update member?").with_default(true);
        if !confirm.prompt()? {
            return Ok(());
        }
    
        let members: Vec<Member> = db.query(&MemberFilter{
            email: Some(update.email.clone()),
            ..Default::default()
        }).await?;
        if members.len() > 0 {
            return Err(anyhow!(
                "Member with email {} already exists.", update.email));
        }

        db.update(update.clone()).await?;

        // If account has changed, create a transaction
        if update.account != member.account {
            let transaction = Transaction{
                member_id: update.id,
                date: datetime::today(),
                amount: update.account - member.account,
                description: format!("Manual account balance update"),
                ..Default::default()
            };
            db.insert(transaction).await?;
        }

        Ok(())
    }
}


#[derive(Args, Debug)]
pub struct DeleteMember{
    #[clap(short, long)]
    pub id: u32,
}


impl DeleteMember {
    pub async fn run(&self, db: &Connection) -> Result<()> {
        let member: Member = db.retrieve(self.id).await?;
        println!("");
        member.print_formatted();
        println!("");
        let confirm = Confirm::new("Delete member from database?")
            .with_default(true);
        if !confirm.prompt()? {
            return Ok(());
        }
        db.delete(member).await?;
        Ok(())
    }
}
