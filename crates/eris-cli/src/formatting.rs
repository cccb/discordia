use eris_accounting::datetime;
use eris_data::Member;

macro_rules! next_attr {
    ($old:ident, $new:ident) => {
        if $old != $new {
            format!(" -> {}", $new)
        } else {
            "".to_string()
        }
    };
    ($old:ident, $new:ident, $attr:ident) => {
        if $old.$attr != $new.$attr {
            format!(" -> {}", $new.$attr)
        } else {
            "".to_string()
        }
    };
}

pub trait PrintFormatted {
    fn print_formatted(&self);
}

impl PrintFormatted for Member {
    fn print_formatted(&self) {
        let memberhip_end = match self.membership_end {
            Some(end) => end.to_string(),
            None => "None".to_string(),
        };

        println!("Name:\t\t\t{}", self.name);
        println!("Email:\t\t\t{}", self.email);
        println!("Notes:\t\t\t{}", self.notes);
        println!("Start:\t\t\t{}", self.membership_start);
        println!("End:\t\t\t{}", memberhip_end);
        println!("Fee:\t\t\t{}", self.fee);
        println!("Interval:\t\t{}", self.interval);
        println!("Last Payment:\t\t{}", self.last_payment);
        println!("Account Balance:\t{}", self.account);
    }
}

impl PrintFormatted for (Member, Member) {
    fn print_formatted(&self) {
        let (old, new) = self;
        let membership_end_old = match old.membership_end {
            Some(end) => end.to_string(),
            None => "None".to_string(),
        };
        let membership_end_new = match new.membership_end {
            Some(end) => end.to_string(),
            None => "None".to_string(),
        };

        let next_name = next_attr!(old, new, name);
        println!("Name:\t\t\t{}{}", old.name, next_name);
        let next_email = next_attr!(old, new, email);
        println!("Email:\t\t\t{}{}", old.email, next_email);
        let next_notes = next_attr!(old, new, notes);
        println!("Notes:\t\t\t{}{}", old.notes, next_notes);
        let next_membership_start = next_attr!(old, new, membership_start);
        println!(
            "Start:\t\t\t{}{}",
            old.membership_start, next_membership_start
        );
        let next_membership_end =
            next_attr!(membership_end_old, membership_end_new);
        println!("End:\t\t\t{}{}", membership_end_old, next_membership_end);
        let next_fee = next_attr!(old, new, fee);
        println!("Fee:\t\t\t{}{}", old.fee, next_fee);
        let next_interval = next_attr!(old, new, interval);
        println!("Interval:\t\t{}{}", old.interval, next_interval);
        let next_last_payment = next_attr!(old, new, last_payment);
        println!("Last Payment:\t\t{}{}", old.last_payment, next_last_payment);
        let next_account = next_attr!(old, new, account);
        println!("Account Balance:\t{}{}", old.account, next_account);
    }
}

impl PrintFormatted for Vec<Member> {
    fn print_formatted(&self) {
        let today = datetime::today();
        println!(
            "{:>4}\t{:<24}\t{:<30}\t{:<24}\t{:>12}\t{}\t{}\t{}\t{}",
            "ID",
            "Name",
            "Email",
            "Notes",
            "Account",
            "Last Payment",
            "Interval",
            "Fee",
            "Inacive"
        );
        println!("{:-<180}", "-");

        for member in self {
            let inactive = if member.is_active(today) { "" } else { "*" };
            println!("{:>4}\t{:<24}\t{:<30}\t{:<24}\t{:>12.2}\t{}\t{:>12}\t{:>}\t{:>}",
                member.id, member.name, member.email,
                member.notes, member.account, member.last_payment,
                member.interval, member.fee, inactive);
        }
    }
}
