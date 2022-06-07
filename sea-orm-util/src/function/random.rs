use sea_orm::sea_query::types::Iden;

pub struct Random;

impl Iden for Random {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        s.write_str("RANDOM").unwrap();
    }
}
