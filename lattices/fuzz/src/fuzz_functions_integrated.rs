#[macro_export]
macro_rules! create_fuzz_functions {
    ($type:ty, $functions:ident) => {
        static $functions: once_cell::sync::Lazy<
            lattices_fuzz::algebra_functions::FuzzFunctions<$type>,
        > = once_cell::sync::Lazy::new(|| {
            lattices_fuzz::algebra_functions::FuzzFunctions::new(
                
| mut x , y | { x *= y ; x %= 1000000 ; x },
Some(|a: $type, b: $type| a.wrapping_mul(b)),
Some(|a: $type| a)
)
});
};
}