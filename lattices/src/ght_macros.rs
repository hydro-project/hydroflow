
/// Helper that does the heavy lifting for GhtType!
#[macro_export]
macro_rules! GhtRowTypeWithSchema {
    // Empty key & Val (Leaf)
    (() => () => $( $schema:ty ),+ ) => (
        $crate::ght::GhtLeaf::<$( $schema ),*,  ()  >
    );

    // Empty key (Leaf)
    (() => $( $z:ty ),* => $schema:ty ) => (
        $crate::ght::GhtLeaf::<$schema,  $crate::variadics::var_type!($( $z ),* ), $crate::variadics::variadic_collections::VariadicCountedHashSet<$schema> >
    );

    // Singleton key & Empty val (Inner over Leaf)
    ($a:ty => () => $schema:ty ) => (
        $crate::ght::GhtInner::<$a, $crate::ght::GhtLeaf::<$schema, (), $crate::variadics::variadic_collections::VariadicCountedHashSet<$schema> >>
    );

    // Singleton key (Inner over Leaf)
    ($a:ty => $( $z:ty ),* => $schema:ty ) => (
        $crate::ght::GhtInner::<$a, $crate::ght::GhtLeaf::<$schema, $crate::variadics::var_type!($( $z ),*), $crate::variadics::variadic_collections::VariadicCountedHashSet<$schema> >>
    );

    // Recursive case with empty val
    ($a:ty, $( $b:ty ),* => () => $schema:ty ) => (
        $crate::ght::GhtInner::<$a, $crate::GhtRowTypeWithSchema!($( $b ),* => () => $schema)>
    );

    // Recursive case.
    ($a:ty, $( $b:ty ),* => $( $z:ty ),* => $schema:ty ) => (
        $crate::ght::GhtInner::<$a, $crate::GhtRowTypeWithSchema!($( $b ),* => $( $z ),* => $schema)>
    );
}

// Hack to test column store: just clones the above using VariadicColumnMultiset instead of VariadicCountedHashSet
// Should unify these macros
/// Helper that does the heavy lifting for GhtType!
#[macro_export]
macro_rules! GhtColumnTypeWithSchema {
    // Empty key & Val (Leaf)
    (() => () => $( $schema:ty ),+ ) => (
        $crate::ght::GhtLeaf::<$( $schema ),*,  ()  >
    );

    // Empty key (Leaf)
    (() => $( $z:ty ),* => $schema:ty ) => (
        $crate::ght::GhtLeaf::<$schema,  $crate::variadics::var_type!($( $z ),* ), $crate::variadics::variadic_collections::VariadicColumnMultiset<$schema> >
    );

    // Singleton key & Empty val (Inner over Leaf)
    ($a:ty => () => $schema:ty ) => (
        $crate::ght::GhtInner::<$a, $crate::ght::GhtLeaf::<$schema, (), $crate::variadics::variadic_collections::VariadicColumnMultiset<$schema> >>
    );

    // Singleton key (Inner over Leaf)
    ($a:ty => $( $z:ty ),* => $schema:ty ) => (
        $crate::ght::GhtInner::<$a, $crate::ght::GhtLeaf::<$schema, $crate::variadics::var_type!($( $z ),*), $crate::variadics::variadic_collections::VariadicColumnMultiset<$schema> >>
    );

    // Recursive case with empty val
    ($a:ty, $( $b:ty ),* => () => $schema:ty ) => (
        $crate::ght::GhtInner::<$a, $crate::GhtColumnTypeWithSchema!($( $b ),* => () => $schema)>
    );

    // Recursive case.
    ($a:ty, $( $b:ty ),* => $( $z:ty ),* => $schema:ty ) => (
        $crate::ght::GhtInner::<$a, $crate::GhtColumnTypeWithSchema!($( $b ),* => $( $z ),* => $schema)>
    );
}

// Hack to test set: just clones the above using VariadicHashSet instead of VariadicCountedHashSet
// Should unify these macros
/// Helper that does the heavy lifting for GhtType!
#[macro_export]
macro_rules! GhtSetTypeWithSchema {
    // Empty key & Val (Leaf)
    (() => () => $( $schema:ty ),+ ) => (
        $crate::ght::GhtLeaf::<$( $schema ),*,  ()  >
    );

    // Empty key (Leaf)
    (() => $( $z:ty ),* => $schema:ty ) => (
        $crate::ght::GhtLeaf::<$schema,  $crate::variadics::var_type!($( $z ),* ), $crate::variadics::variadic_collections::VariadicHashSet<$schema> >
    );

    // Singleton key & Empty val (Inner over Leaf)
    ($a:ty => () => $schema:ty ) => (
        $crate::ght::GhtInner::<$a, $crate::ght::GhtLeaf::<$schema, (), $crate::variadics::variadic_collections::VariadicHashSet<$schema> >>
    );

    // Singleton key (Inner over Leaf)
    ($a:ty => $( $z:ty ),* => $schema:ty ) => (
        $crate::ght::GhtInner::<$a, $crate::ght::GhtLeaf::<$schema, $crate::variadics::var_type!($( $z ),*), $crate::variadics::variadic_collections::VariadicHashSet<$schema> >>
    );

    // Recursive case with empty val
    ($a:ty, $( $b:ty ),* => () => $schema:ty ) => (
        $crate::ght::GhtInner::<$a, $crate::GhtSetTypeWithSchema!($( $b ),* => () => $schema)>
    );

    // Recursive case.
    ($a:ty, $( $b:ty ),* => $( $z:ty ),* => $schema:ty ) => (
        $crate::ght::GhtInner::<$a, $crate::GhtSetTypeWithSchema!($( $b ),* => $( $z ),* => $schema)>
    );
}

/// Macro to construct a Ght node type from the constituent key and
/// dependent column types. You pass it:
///    - a list of key column types and dependent column type separated by a fat arrow,
///         a la (K1, K2, K3 => T1, T2, T3)
///
/// This macro generates a hierarchy of GHT node types where each key column is associated with an GhtInner
/// of the associated column type, and the remaining dependent columns are associated with a variadic HTleaf
/// a la var_expr!(T1, T2, T3)
#[macro_export]
macro_rules! GhtType {
    // Empty key
    (() => $( $z:ty ),*: Row ) => (
        $crate::GhtRowTypeWithSchema!(() => $( $z ),* => $crate::variadics::var_type!($( $z ),* ))
    );

    (() => $( $z:ty ),*: Column ) => (
        $crate::GhtColumnTypeWithSchema!(() => $( $z ),* => $crate::variadics::var_type!($( $z ),* ))
    );

    (() => $( $z:ty ),*: Set ) => (
        $crate::GhtSetTypeWithSchema!(() => $( $z ),* => $crate::variadics::var_type!($( $z ),* ))
    );

    // Recursive case empty val
    ($( $b:ty ),* => (): Row ) => (
        $crate::GhtRowTypeWithSchema!($( $b ),* => () => $crate::variadics::var_type!($( $b ),*))
    );

    ($( $b:ty ),* => (): Column ) => (
        $crate::GhtColumnTypeWithSchema!($( $b ),* => () => $crate::variadics::var_type!($( $b ),*))
    );

    ($( $b:ty ),* => (): Set ) => (
        $crate::GhtSetTypeWithSchema!($( $b ),* => () => $crate::variadics::var_type!($( $b ),*))
    );

    // Recursive case
    ($( $b:ty ),* => $( $z:ty ),*: Row) => (
        $crate::GhtRowTypeWithSchema!($( $b ),* => $( $z ),* => $crate::variadics::var_type!($( $b ),*, $( $z ),*))
    );

    ($( $b:ty ),* => $( $z:ty ),*: Column) => (
        $crate::GhtColumnTypeWithSchema!($( $b ),* => $( $z ),* => $crate::variadics::var_type!($( $b ),*, $( $z ),*))
    );

    ($( $b:ty ),* => $( $z:ty ),*: Set) => (
        $crate::GhtSetTypeWithSchema!($( $b ),* => $( $z ),* => $crate::variadics::var_type!($( $b ),*, $( $z ),*))
    );
}

#[macro_export]
/// Constructs a forest (variadic list) of Ght structs,
/// one for each height from 0 to length of the schema - 1
macro_rules! GhtForestType {
    // 1 => 0
    ($a:ty => ()) => {
        var_type!($crate::GhtType!($a => (): Column))
    };
    // 1 => 1
    ($a:ty => $c:ty ) => {
        ($crate::GhtType!($a => $c: Column), GhtForestType!($a, $c => ()))
    };
    // 1 => >1
    ($a:ty => $c:ty, $( $d:ty ),* ) => {
        ($crate::GhtType!($a => $c, $( $d ),*: Column), GhtForestType!($a, $c => $( $d ),*))
    };
    // >1 => 0
    ($a:ty, $( $b:ty ),* => ()) => {
        var_type!($crate::GhtType!($a, $( $b ),* => (): Column))
    };
    // >1 => 1
    ($a:ty, $( $b:ty ),* => $c:ty) => {
        ($crate::GhtType!($a, $( $b ),* => $c: Column), GhtForestType!($a, $( $b ),*, $c => ()))
    };
    // >1 => >1
    ($a:ty, $( $b:ty ),* => $c:ty, $( $d:ty ),* ) => {
        ($crate::GhtType!($a, $( $b ),* => $c, $( $d ),*: Column), GhtForestType!($a, $( $b ),* , $c => $( $d ),*))
    };
    // general 1
    ($a:ty) => {
        ($crate::GhtType!(() => $a: Column), GhtForestType!($a => ()))
    };
    // general >1
    ($a:ty, $( $b:ty ),* ) => {
        ($crate::GhtType!(() => $a, $( $b ),*: Column), GhtForestType!($a => $( $b ),*))
    };
}