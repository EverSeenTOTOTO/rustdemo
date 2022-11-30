#[macro_export]
macro_rules! fib {
    (0) => {1};
    (1) => {1};
    ($num:expr) => {
        fib!($num - 1) + fib!($num - 2)
    }
}
