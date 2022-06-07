#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(r#"length of "{0}" more than {1}"#)]
    MoreThan(String, usize),
    #[error(r#"length of "{0}" less than {1}"#)]
    LessThan(String, usize),
    #[error(r#""{0}" is not email"#)]
    NotEmail(String),
}

#[derive(Debug)]
pub struct StringValidator(Result<String, Error>);

impl StringValidator {
    pub fn new(x: String) -> Self {
        Self(Ok(x))
    }

    pub fn take(self) -> Result<String, Error> {
        self.0
    }

    pub fn max(self, n: usize) -> Self {
        match self.0 {
            Ok(x) if x.chars().count() > n => Self(Err(Error::MoreThan(x, n))),
            _ => self,
        }
    }

    pub fn min(self, n: usize) -> Self {
        match self.0 {
            Ok(x) if x.chars().count() < n => Self(Err(Error::LessThan(x, n))),
            _ => self,
        }
    }

    pub fn email(self) -> Self {
        fn is_not_email(st: &str) -> bool {
            let mut r = st.split('@');

            let has_user = r.next().map(|x| x.chars().count() > 0).unwrap_or(false);
            let has_domain = r.next().map(|x| x.chars().count() > 0).unwrap_or(false);

            !(has_user && has_domain)
        }

        match self.0 {
            Ok(x) if is_not_email(&x) => Self(Err(Error::NotEmail(x))),
            _ => self,
        }
    }
}

#[test]
fn test() {
    use super::ValidatorStringExt;

    let _r = "aa".validate().max(2).take().unwrap();
    let _r = "aa".validate().min(2).take().unwrap();
    let _r = "a@a".validate().email().take().unwrap();

    let r = "aa"
        .validate()
        .max(1)
        .take()
        .expect_err("exepcted err, but returns ok");

    match r {
        Error::MoreThan(x, n) if x.chars().count() > n => {}
        _ => panic!(),
    }

    let r = "a"
        .validate()
        .min(2)
        .take()
        .expect_err("expected err, but returns ok");

    match r {
        Error::LessThan(x, n) if x.chars().count() < n => {}
        _ => panic!(),
    }

    let r = "@"
        .validate()
        .email()
        .take()
        .expect_err("expected err, but returns ok");

    match r {
        Error::NotEmail(_) => {}
        _ => panic!(),
    }
}
