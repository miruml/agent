// standard library
use std::fmt::Debug;
// use backtrace::Backtrace;
// external crates
#[allow(unused_imports)]
use tracing::{error, info, trace, warn};

pub trait MiruError
where
    Self: Debug,
{
    fn network_connection_error(&self) -> bool;
}

pub fn are_all_network_connection_errors<I>(errors: I) -> bool
where
    I: IntoIterator,
    I::Item: AsRef<dyn MiruError>,
{
    errors
        .into_iter()
        .all(|e| e.as_ref().network_connection_error())
}

#[derive(Debug, Clone)]
pub struct Trace {
    pub file: &'static str,
    pub line: u32,
    // pub stack_trace: Backtrace,
}

#[macro_export]
macro_rules! trace {
    () => {
        Box::new($crate::errors::Trace {
            file: file!(),
            line: line!(),
            // stack_trace: backtrace::Backtrace::new(),
        })
    };
}

#[macro_export]
macro_rules! deserialize_error {
    ($struct_name:expr, $field_name:expr, $default:expr) => {{
        error!(
            "'{}' missing from struct '{}', setting to default: '{:?}'",
            $field_name, $struct_name, $default
        );
        $default
    }};
}

#[macro_export]
macro_rules! deserialize_warn {
    ($struct_name:expr, $field_name:expr, $default:expr) => {{
        warn!(
            "'{}' missing from struct '{}', setting to default: '{:?}'",
            $field_name, $struct_name, $default
        );
        $default
    }};
}
