#[macro_export]
macro_rules! rassert {
    ($cond:expr $(,)?) => {
        $crate::rassert!($cond, "assertion failed: `{}`", stringify!($cond))
    };
    ($cond:expr, $fmt:literal) => {
        $crate::rassert!($cond, $fmt,)
    };
    ($cond:expr, $fmt:literal, $($arg:tt)*) => {
        {
            if $cond {
                Ok(())
            }
            else {
                Err(format!($fmt, $($arg)*))
            }
        }
    };
}

#[macro_export]
macro_rules! rassert_eq {
    ($a:expr, $b:expr) => {
        $crate::rassert!($a == $b,)
    };
    ($a:expr, $b:expr, $($arg:tt)*) => {
        $crate::rassert!($a == $b, $($arg)*)
    };
}
