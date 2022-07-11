use std::{
    fmt::Display,
    ops::{Bound, RangeBounds},
};

pub const RANGE_0: &'static dyn RangeTrait<usize> = &(0..=0);
pub const RANGE_1: &'static dyn RangeTrait<usize> = &(1..=1);

pub const OPERATORS: [OperatorConstraints; 8] = [
    OperatorConstraints {
        name: "merge",
        hard_range_inn: &(0..),
        soft_range_inn: &(2..),
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        write_fn: &(|| {}),
    },
    OperatorConstraints {
        name: "join",
        hard_range_inn: &(2..=2),
        soft_range_inn: &(2..=2),
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        write_fn: &(|| {}),
    },
    OperatorConstraints {
        name: "tee",
        hard_range_inn: RANGE_1,
        soft_range_inn: RANGE_1,
        hard_range_out: &(0..),
        soft_range_out: &(2..),
        write_fn: &(|| {}),
    },
    OperatorConstraints {
        name: "map",
        hard_range_inn: RANGE_1,
        soft_range_inn: RANGE_1,
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        write_fn: &(|| {}),
    },
    OperatorConstraints {
        name: "dedup",
        hard_range_inn: RANGE_1,
        soft_range_inn: RANGE_1,
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        write_fn: &(|| {}),
    },
    OperatorConstraints {
        name: "input",
        hard_range_inn: RANGE_0,
        soft_range_inn: RANGE_0,
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        write_fn: &(|| {}),
    },
    OperatorConstraints {
        name: "seed",
        hard_range_inn: RANGE_0,
        soft_range_inn: RANGE_0,
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        write_fn: &(|| {}),
    },
    OperatorConstraints {
        name: "for_each",
        hard_range_inn: RANGE_1,
        soft_range_inn: RANGE_1,
        hard_range_out: RANGE_0,
        soft_range_out: RANGE_0,
        write_fn: &(|| {}),
    },
];

pub struct OperatorConstraints {
    pub name: &'static str,
    pub hard_range_inn: &'static dyn RangeTrait<usize>,
    pub soft_range_inn: &'static dyn RangeTrait<usize>,
    pub hard_range_out: &'static dyn RangeTrait<usize>,
    pub soft_range_out: &'static dyn RangeTrait<usize>,
    pub write_fn: &'static dyn Fn(),
}

pub trait RangeTrait<T>
where
    T: ?Sized,
{
    fn start_bound(&self) -> Bound<&T>;
    fn end_bound(&self) -> Bound<&T>;
    fn contains(&self, item: &T) -> bool
    where
        T: PartialOrd<T>;

    fn human_string(&self) -> String
    where
        T: Display + PartialEq,
    {
        match (self.start_bound(), self.end_bound()) {
            (Bound::Unbounded, Bound::Unbounded) => "any number of".to_owned(),

            (Bound::Included(n), Bound::Included(x)) if n == x => {
                format!("exactly {}", n)
            }
            (Bound::Included(n), Bound::Included(x)) => {
                format!("at least {} and at most {}", n, x)
            }
            (Bound::Included(n), Bound::Excluded(x)) => {
                format!("at least {} and less than {}", n, x)
            }
            (Bound::Included(n), Bound::Unbounded) => format!("at least {}", n),
            (Bound::Excluded(n), Bound::Included(x)) => {
                format!("more than {} and at most {}", n, x)
            }
            (Bound::Excluded(n), Bound::Excluded(x)) => {
                format!("more than {} and less than {}", n, x)
            }
            (Bound::Excluded(n), Bound::Unbounded) => format!("more than {}", n),
            (Bound::Unbounded, Bound::Included(x)) => format!("at most {}", x),
            (Bound::Unbounded, Bound::Excluded(x)) => format!("less than {}", x),
        }
    }
}

impl<R, T> RangeTrait<T> for R
where
    R: RangeBounds<T>,
{
    fn start_bound(&self) -> Bound<&T> {
        self.start_bound()
    }

    fn end_bound(&self) -> Bound<&T> {
        self.end_bound()
    }

    fn contains(&self, item: &T) -> bool
    where
        T: PartialOrd<T>,
    {
        self.contains(item)
    }
}
