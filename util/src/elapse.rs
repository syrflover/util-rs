#[macro_export]
macro_rules! elapse {
    ($label:expr, $expr:expr) => {{
        #[cfg(debug_assertions)]
        {
            let start = ::std::time::SystemTime::now();

            let expr = $expr;

            let end = start
                .elapsed()
                .as_ref()
                .map(::std::time::Duration::as_micros)
                .unwrap();

            ::log::debug!("{}: {}ms", $label, end as f64 / 1000.0);

            expr
        }

        #[cfg(not(debug_assertions))]
        {
            $expr
        }
    }};
}

#[allow(unused_imports)]
pub(crate) use elapse;

#[cfg(test)]
mod tests {
    #[test]
    fn test_elapse() {
        let _r = elapse!("label", "str");
    }
}
