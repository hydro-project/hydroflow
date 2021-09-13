#[derive(Debug, Clone, Copy, Default)]
#[derive(PartialOrd, Ord, PartialEq, Eq)]
pub struct OpProps {
    pub complete: bool,
    pub time_ordered: bool,
    pub lattice_ordered: bool, // E.g. cumulative.
}

impl OpProps {
    pub const fn default() -> Self {
        Self {
            complete: false,
            time_ordered: false,
            lattice_ordered: false,
        }
    }
    pub const fn combine(a: Self, b: Self) -> Self {
        Self {
            complete: a.complete || b.complete,
            time_ordered: a.time_ordered || b.time_ordered,
            lattice_ordered: a.lattice_ordered || b.lattice_ordered,
        }
    }
}

pub const fn require_lattice_ordered(complete: bool, time_ordered: bool) -> OpProps {
    OpProps {
        complete, time_ordered,
        lattice_ordered: true,
    }
}

// MACRO-GENERATED (TODO):
#[macro_export]
macro_rules! generate_ops_helper {
    (
        $name:ident,
        $mapper:ident,
        $( ($a:literal, $b:literal, $c:literal), )*
    ) => {
        $(
            #[doc = "- COMPLETE: "]
            #[doc = stringify!($a)]
            #[doc = "\n- TIME_ORDERED: "]
            #[doc = stringify!($b)]
            #[doc = "\n- LATTICE_ORDERED: "]
            #[doc = stringify!($c)]
            #[allow(trivial_bounds)]
            impl Op<{OpProps { complete: $a, time_ordered: $b, lattice_ordered: $c }}> for $name
            where
                Self: OpImpl<{OpProps::combine($mapper(OpProps { complete: $a, time_ordered: $b, lattice_ordered: $c }), OpProps { complete: $a, time_ordered: $b, lattice_ordered: $c })}>,
            {
                fn get() {
                    <$name as OpImpl<{OpProps::combine($mapper(OpProps { complete: $a, time_ordered: $b, lattice_ordered: $c }), OpProps { complete: $a, time_ordered: $b, lattice_ordered: $c })}>>::get()
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! generate_ops {
    ($name:ident, $mapper:ident) => {
        generate_ops_helper!(
            $name,
            $mapper,
            (false, false, false),
            (false, false,  true),
            (false,  true, false),
            (false,  true,  true),
            ( true, false, false),
            ( true, false,  true),
            ( true,  true, false),
            ( true,  true,  true),
        );
    };
}
pub use generate_ops;
