/// General helper functions and macros for internal use;
/// these should not be exposed externally.

/// A macro which takes a string and wraps it as an io Error of the `Other`
/// `ErrorKind`. So writing `io_err!(input_string)` is the same as:
/// ```
/// Err(
///     io::Error::new(
///         io::ErrorKind::Other, input_string
///     )
/// )
/// ```
/// 
/// This can also accept a format string; ex: `io_err!("Error code: {}", 30)`
#[doc(hidden)]
#[macro_export]
macro_rules! io_err {
    // simple strings
    ($msg:expr) => {
        Err(std::io::Error::new(std::io::ErrorKind::Other, $msg))
    };

    // support format strings
    ($fmt:expr, $($arg:tt)*) => {
        Err(std::io::Error::new(std::io::ErrorKind::Other, format!($fmt, $($arg)*)))
    };
}