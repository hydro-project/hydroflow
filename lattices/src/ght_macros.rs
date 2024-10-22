#[macro_export]
/// Internal macro for constructing a Ght struct with the given schema and storage type
///
/// Should not be used directly, use `GhtType!` instead
macro_rules! GhtTypeWithSchema {
    // Empty key & Val (Leaf)
    (() => () => $( $schema:ty ),+ : $storage:ident) => (
        $crate::ght::GhtLeaf::<$( $schema ),*,  ()  >
    );

    // Empty key (Leaf)
    (() => $( $z:ty ),* => $schema:ty : $storage:ident) => (
        $crate::ght::GhtLeaf::<$schema,  $crate::variadics::var_type!($( $z ),*), $crate::variadics::variadic_collections::$storage<$schema> >
    );

    // Singleton key & Empty val (Inner over Leaf)
    ($a:ty => () => $schema:ty : $storage:ident) => (
        $crate::ght::GhtInner::<$a, $crate::ght::GhtLeaf::<$schema, (), $crate::variadics::variadic_collections::$storage<$schema> >>
    );

    // Singleton key (Inner over Leaf)
    ($a:ty => $( $z:ty ),* => $schema:ty : $storage:ident) => (
        $crate::ght::GhtInner::<$a, $crate::ght::GhtLeaf::<$schema, $crate::variadics::var_type!($( $z ),*), $crate::variadics::variadic_collections::$storage<$schema> >>
    );

    // Recursive case with empty val
    ($a:ty, $( $b:ty ),* => () => $schema:ty : $storage:ident) => (
        $crate::ght::GhtInner::<$a, $crate::GhtTypeWithSchema!($( $b ),* => () => $schema : $storage)>
    );

    // Recursive case
    ($a:ty, $( $b:ty ),* => $( $z:ty ),* => $schema:ty : $storage:ident) => (
        $crate::ght::GhtInner::<$a, $crate::GhtTypeWithSchema!($( $b ),* => $( $z ),* => $schema : $storage)>
    );
}

#[macro_export]
/// Public macro for constructing a Ght struct with the given schema and storage type
///
/// # Example
/// ```
/// use lattices::GhtType;
/// use variadics::variadic_collections::VariadicHashSet;
///
/// type MyGht = GhtType!(u16, u32 => u64: VariadicHashSet);
/// ```
macro_rules! GhtType {
    // Empty key
    (() => $( $z:ty ),*: $storage:ident) => (
        $crate::GhtTypeWithSchema!(() => $( $z ),* => $crate::variadics::var_type!($( $z ),*): $storage)
    );

    // Recursive case empty val
    ($( $b:ty ),* => (): $storage:ident) => (
        $crate::GhtTypeWithSchema!($( $b ),* => () => $crate::variadics::var_type!($( $b ),*): $storage)
    );

    // Recursive case
    ($( $b:ty ),* => $( $z:ty ),*: $storage:ident) => (
        $crate::GhtTypeWithSchema!($( $b ),* => $( $z ),* => $crate::variadics::var_type!($( $b ),*, $( $z ),*): $storage)
    );
}

#[macro_export]
/// Construct a forest of Ghts (i.e. a ColtForest) with the given schema and storage type.
///
/// # Example
/// ```
/// use lattices::ColtType;
///
/// type MyColt = ColtType!(u16, u32, u64);
/// ```

macro_rules! ColtType {
    // Base case: single type to empty
    ($a:ty => ()) => {
        $crate::variadics::var_type!($crate::GhtType!($a => (): VariadicColumnMultiset))
    };
    // Base case: single type to single type
    ($a:ty => $c:ty) => {
        ($crate::GhtType!($a => $c: VariadicColumnMultiset), ColtType!($a, $c => ()))
    };
    // Recursive case: single type to multiple types
    ($a:ty => $c:ty, $( $d:ty ),*) => {
        ($crate::GhtType!($a => $c, $( $d ),*: VariadicColumnMultiset), ColtType!($a, $c => $( $d ),*))
    };
    // Base case: multiple types to empty
    ($a:ty, $( $b:ty ),* => ()) => {
        $crate::variadics::var_type!($crate::GhtType!($a, $( $b ),* => (): VariadicColumnMultiset))
    };
    // Base case: multiple types to single type
    ($a:ty, $( $b:ty ),* => $c:ty) => {
        ($crate::GhtType!($a, $( $b ),* => $c: VariadicColumnMultiset), ColtType!($a, $( $b ),*, $c => ()))
    };
    // Recursive case: multiple types to multiple types
    ($a:ty, $( $b:ty ),* => $c:ty, $( $d:ty ),*) => {
        ($crate::GhtType!($a, $( $b ),* => $c, $( $d ),*: VariadicColumnMultiset), ColtType!($a, $( $b ),*, $c => $( $d ),*))
    };
    // General case: single type
    ($a:ty) => {
        ($crate::GhtType!(() => $a: VariadicColumnMultiset), ColtType!($a => ()))
    };
    // General case: multiple types
    ($a:ty, $( $b:ty ),*) => {
        ($crate::GhtType!(() => $a, $( $b ),*: VariadicColumnMultiset), ColtType!($a => $( $b ),*))
    };
}
