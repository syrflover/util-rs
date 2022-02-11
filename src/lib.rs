pub mod r#async;
mod from_request;
pub mod http;
mod read_chunks;
pub mod validate;

pub use from_request::{FromRequest, IntoPayload};
pub use read_chunks::ReadChunks;

#[macro_export]
macro_rules! tri {
    ($expr:expr) => {
        match $expr {
            Ok(v) => v,
            Err(error) => return error.into(),
        }
    };
}

#[macro_export]
macro_rules! ori {
    ($expr:expr) => {
        match $expr {
            Some(v) => v,
            None => return Ok(None),
        }
    };
}

#[macro_export]
macro_rules! test_registry {
    ([$($ap:ident: $apty:ty),*] -> [$($p:ident: $pty:ty),*] -> $ready:expr, $test:expr) => {
        test_registry!([(Arc) -> $($ap: $apty),*] -> [$($p: $pty),*] -> $ready, $test)
    };

    ([(Arc) -> $($ap:ident: $apty:ty),*] -> [$($p:ident: $pty:ty),*] -> $ready:expr, $test:expr) => {
        ::sai::component_registry!(TestRegistry, [Test, Ready]);

        #[derive(::sai::Component)]
        #[lifecycle]
        struct Ready {
            $(
                pub $ap: ::std::sync::Arc<$apty>,
            )*
            $(
                pub $p: $pty,
            )*
        }

        #[async_trait::async_trait]
        impl ::sai::ComponentLifecycle for Ready {
            #[allow(unused_assignments, unused_mut, unused_variables)]
            async fn start(&mut self) {
                $(
                    let mut $ap: $apty = Default::default();
                )*
                $(
                    let mut $p = self.$p.clone();
                )*

                $ready

                $(
                    self.$ap = ::std::sync::Arc::new($ap);
                )*
                $(
                    self.$p = $p;
                )*
            }
        }

        #[derive(::sai::Component)]
        #[lifecycle]
        struct Test {
            #[injected]
            ready: ::sai::Injected<Ready>,
        }

        #[async_trait::async_trait]
        impl ::sai::ComponentLifecycle for Test {
            #[allow(dead_code, unused_variables)]
            async fn start(&mut self) {
                let ready = ::std::sync::Arc::clone(&self.ready);

                $(
                    let $ap = ::std::sync::Arc::clone(&ready.$ap);
                )*
                $(
                    let $p = ready.$p.clone();
                )*

                $test
            }
        }
    };

    ([(Injected) -> $($ap:ident: $apty:ty),*] -> [$($p:ident: $pty:ty),*] -> $ready:expr, $test:expr) => {
        ::sai::component_registry!(TestRegistry, [Test, Ready]);

        #[derive(::sai::Component)]
        #[lifecycle]
        struct Ready {
            $(
                pub $ap: ::sai::Injected<$apty>,
            )*
            $(
                pub $p: $pty,
            )*
        }

        #[async_trait::async_trait]
        impl ::sai::ComponentLifecycle for Ready {
            #[allow(unused_assignments, unused_mut, unused_variables)]
            async fn start(&mut self) {
                $(
                    let mut $ap: $apty = Default::default();
                )*
                $(
                    let mut $p = self.$p.clone();
                )*

                $ready

                $(
                    self.$ap = ::sai::Injected::new($ap);
                )*
                $(
                    self.$p = $p;
                )*
            }
        }

        #[derive(::sai::Component)]
        #[lifecycle]
        struct Test {
            #[injected]
            ready: ::sai::Injected<Ready>,
        }

        #[async_trait::async_trait]
        impl ::sai::ComponentLifecycle for Test {
            #[allow(dead_code, unused_variables)]
            async fn start(&mut self) {
                let ready = ::sai::Injected::clone(&self.ready);

                $(
                    let $ap = ::sai::Injected::clone(&ready.$ap);
                )*
                $(
                    let $p = ready.$p.clone();
                )*

                $test
            }
        }
    };
}

#[macro_export]
macro_rules! assert_debug {
    ($left:expr, $right:expr $(,)?) => {
        let left = format!("{:?}", $left);
        let right = format!("{:?}", $right);
        assert_eq!(left, right);
    };

    (($left:expr, $right:expr, $($arg:tt)+)) => {
        let left = format!("{:?}", $left);
        let right = format!("{:?}", $right);
        assert_eq!(left, right, $($arg)+);
    };
}
