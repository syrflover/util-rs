pub mod number;
pub mod string;

pub use number::*;
pub use string::*;

pub trait ValidatorStringExt {
    fn validate(self) -> StringValidator;
}

impl ValidatorStringExt for String {
    fn validate(self) -> StringValidator {
        StringValidator::new(self)
    }
}

impl ValidatorStringExt for &str {
    fn validate(self) -> StringValidator {
        StringValidator::new(self.to_owned())
    }
}

pub trait ValidatorNumberExt: Num {
    fn validate(self) -> NumberValidator<Self>;
}

macro_rules! impl_validate_number_ext {
    ($($n:ty),*) => {
        $(
            impl ValidatorNumberExt for $n {
                fn validate(self) -> NumberValidator<Self> {
                    NumberValidator::<$n>::new(self)
                }
            }
        )*
    };
}

impl_validate_number_ext![u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize];

pub enum Error {
    String(),
    Number(),
}
