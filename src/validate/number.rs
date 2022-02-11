use std::fmt::{Debug, Display};

#[derive(Debug, thiserror::Error)]
pub enum Error<T: Num> {
    /// (Original, N)
    #[error("{0} more than {1}")]
    MoreThan(T, T),
    /// (Original, N)
    #[error("{0} less than {1}")]
    LessThan(T, T),
    /// (Original, N)
    #[error("{0} not equal {1}")]
    NotEqual(T, T),
}

#[derive(Debug)]
pub struct NumberValidator<T: Num>(Result<T, Error<T>>);

impl<T: Num> NumberValidator<T> {
    pub fn new(x: T) -> Self {
        Self(Ok(x))
    }

    pub fn take(self) -> Result<T, Error<T>> {
        self.0
    }

    pub fn max(self, n: T) -> Self {
        match self.0 {
            Ok(x) if x > n => Self(Err(Error::MoreThan(x, n))),
            _ => self,
        }
    }

    pub fn min(self, n: T) -> Self {
        match self.0 {
            Ok(x) if x < n => Self(Err(Error::LessThan(x, n))),
            _ => self,
        }
    }

    pub fn eq(self, n: T) -> Self {
        match self.0 {
            Ok(x) if x != n => Self(Err(Error::NotEqual(x, n))),
            _ => self,
        }
    }
}

#[test]
fn test() {
    use super::ValidatorNumberExt;

    let _r = (2).validate().max(2).take().unwrap();
    let _r = (2).validate().min(2).take().unwrap();
    let _r = (2).validate().eq(2).take().unwrap();

    let r = (2)
        .validate()
        .max(1)
        .take()
        .expect_err("expected err, but returns ok");

    match r {
        Error::MoreThan(x, n) if x >= n => {}
        _ => panic!(),
    }

    let r = (2)
        .validate()
        .min(3)
        .take()
        .expect_err("expected err, but returns ok");

    match r {
        Error::LessThan(x, n) if x <= n => {}
        _ => panic!(),
    }

    let r = (2)
        .validate()
        .eq(1)
        .take()
        .expect_err("expected err, but returns ok");

    match r {
        Error::NotEqual(x, n) if x != n => {}
        _ => panic!(),
    }
}

pub trait Num: PartialEq + PartialOrd + Copy + Clone + Debug + Default + Display {}

macro_rules! impl_num {
    ($($n:ty),*) => {
        $(
            impl Num for $n {}
        )*
    };
}

impl_num![u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize];
