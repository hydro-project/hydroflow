use core::cmp::Ordering;
use core::convert::Infallible;
use core::fmt::{self, Debug, Display};

use serde::{Deserialize, Serialize};

use crdts::{CmRDT, CvRDT, Dot, DotRange};

pub trait EResetRemove {
    /// Remove data that is strictly smaller than this clock
    fn reset_remove(&mut self, clock: &EVClock);
}

/// A `EVClock` is a standard vector clock.
/// It contains a set of "actors" and associated counters.
/// When a particular actor witnesses a mutation, their associated
/// counter in a `EVClock` is incremented. `EVClock` is typically used
/// as metadata for associated application data, rather than as the
/// container for application data. `EVClock` just tracks causality.
/// It can tell you if something causally descends something else,
/// or if different replicas are "concurrent" (were mutated in
/// isolation, and need to be resolved externally).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EVClock {
    /// dots is the mapping from actors to their associated counters
    pub dots: [u64; 32],
}

impl Default for EVClock {
    fn default() -> Self {
        Self { dots: [0; 32] }
    }
}

impl PartialOrd for EVClock {
    fn partial_cmp(&self, other: &EVClock) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if other
            .dots
            .iter()
            .enumerate()
            .all(|(w, c)| self.get(&w) >= *c)
        {
            Some(Ordering::Greater)
        } else if self
            .dots
            .iter()
            .enumerate()
            .all(|(w, c)| other.get(&w) >= *c)
        {
            Some(Ordering::Less)
        } else {
            None
        }
    }
}

impl Display for EVClock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<")?;
        for (i, (actor, count)) in self.dots.iter().enumerate().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}:{}", actor, count)?;
        }
        write!(f, ">")
    }
}

impl EResetRemove for EVClock {
    /// Forget any actors that have smaller counts than the
    /// count in the given EVClock
    fn reset_remove(&mut self, other: &Self) {
        for Dot { actor, counter } in other.iter() {
            if counter >= self.get(&actor) {
                self.dots[actor] = 0;
            }
        }
    }
}

impl CmRDT for EVClock {
    type Op = Dot<usize>;
    type Validation = DotRange<usize>;

    fn validate_op(&self, dot: &Self::Op) -> Result<(), Self::Validation> {
        let next_counter = self.get(&dot.actor) + 1;
        if dot.counter > next_counter {
            Err(DotRange {
                actor: dot.actor.clone(),
                counter_range: next_counter..dot.counter,
            })
        } else {
            Ok(())
        }
    }

    /// Monotonically adds the given actor version to
    /// this EVClock.
    ///
    /// # Examples
    /// ```
    /// use crdts::{EVClock, Dot, CmRDT};
    /// let mut v = EVClock::new();
    ///
    /// v.apply(Dot::new("A", 2));
    ///
    /// // now all dots applied to `v` from actor `A` where
    /// // the counter is not bigger than 2 are nops.
    /// v.apply(Dot::new("A", 0));
    /// assert_eq!(v.get(&"A"), 2);
    /// ```
    fn apply(&mut self, dot: Self::Op) {
        if self.get(&dot.actor) < dot.counter {
            self.dots[dot.actor] = dot.counter;
        }
    }
}

impl CvRDT for EVClock {
    type Validation = Infallible;

    fn validate_merge(&self, _other: &Self) -> Result<(), Self::Validation> {
        Ok(())
    }

    fn merge(&mut self, other: Self) {
        for dot in other.iter() {
            self.apply(dot);
        }
    }
}

impl EVClock {
    /// Returns a new `EVClock` instance.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns a clone of self but with information that is older than given clock is
    /// forgotten
    pub fn clone_without(&self, base_clock: &EVClock) -> EVClock {
        let mut cloned = self.clone();
        cloned.reset_remove(base_clock);
        cloned
    }

    /// Generate Op to increment an actor's counter.
    ///
    /// # Examples
    /// ```
    /// use crdts::{EVClock, CmRDT};
    /// let mut a = EVClock::new();
    ///
    /// // `a.inc()` does not mutate the EVClock!
    /// let op = a.inc("A");
    /// assert_eq!(a, EVClock::new());
    ///
    /// // we must apply the op to the EVClock to have
    /// // its edit take effect.
    /// a.apply(op.clone());
    /// assert_eq!(a.get(&"A"), 1);
    ///
    /// // Op's can be replicated to another node and
    /// // applied to the local state there.
    /// let mut other_node = EVClock::new();
    /// other_node.apply(op);
    /// assert_eq!(other_node.get(&"A"), 1);
    /// ```
    pub fn inc(&self, actor: usize) -> Dot<usize> {
        self.dot(actor).inc()
    }

    /// Return the associated counter for this actor.
    /// All actors not in the EVClock have an implied count of 0
    pub fn get(&self, actor: &usize) -> u64 {
        self.dots.get(*actor).cloned().unwrap_or(0)
    }

    /// Return the Dot for a given actor
    pub fn dot(&self, actor: usize) -> Dot<usize> {
        let counter = self.get(&actor);
        Dot::new(actor, counter)
    }

    /// True if two vector clocks have diverged.
    ///
    /// # Examples
    /// ```
    /// use crdts::{EVClock, CmRDT};
    /// let (mut a, mut b) = (EVClock::new(), EVClock::new());
    /// a.apply(a.inc("A"));
    /// b.apply(b.inc("B"));
    /// assert!(a.concurrent(&b));
    /// ```
    pub fn concurrent(&self, other: &EVClock) -> bool {
        self.partial_cmp(other).is_none()
    }

    /// Returns `true` if this vector clock contains nothing.
    pub fn is_empty(&self) -> bool {
        self.dots.is_empty()
    }

    /// Returns the common elements (same actor and counter)
    /// for two `EVClock` instances.
    pub fn intersection(left: &EVClock, right: &EVClock) -> EVClock {
        let mut dots = [0u64; 32];
        for (left_actor, left_counter) in left.dots.iter().enumerate() {
            let right_counter = right.get(&left_actor);
            if right_counter == *left_counter {
                dots[left_actor] = *left_counter;
            }
        }

        Self { dots }
    }

    // /// Reduces this EVClock to the greatest-lower-bound of the given
    // /// EVClock and itsef, as an example see the following code.
    // /// ``` rust
    // /// use crdts::{EVClock, Dot, ResetRemove, CmRDT};
    // /// let mut c = EVClock::new();
    // /// c.apply(Dot::new(23, 6));
    // /// c.apply(Dot::new(89, 14));
    // /// let c2 = c.clone();
    // ///
    // /// c.glb(&c2); // this is a no-op since `glb { c, c } = c`
    // /// assert_eq!(c, c2);
    // ///
    // /// c.apply(Dot::new(43, 1));
    // /// assert_eq!(c.get(&43), 1);
    // /// c.glb(&c2); // should remove the 43 => 1 entry
    // /// assert_eq!(c.get(&43), 0);
    // /// ```
    // pub fn glb(&mut self, other: &Self) {
    //     self.dots = mem::take(&mut self.dots)
    //         .into_iter()
    //         .enumerate()
    //         .filter_map(|(actor, count)| {
    //             // Since an actor missing from the dots map has an implied
    //             // counter of 0 we can save some memory, and remove the actor.
    //             let min_count = cmp::min(count, other.get(&actor));
    //             match min_count {
    //                 0 => None,
    //                 _ => Some((actor, min_count)),
    //             }
    //         })
    //         .collect();
    // }

    /// Returns an iterator over the dots in this EVClock
    pub fn iter(&self) -> impl Iterator<Item = Dot<usize>> + '_ {
        self.dots.iter().enumerate().map(|(a, c)| Dot {
            actor: a,
            counter: *c,
        })
    }
}

// pub struct IntoIter {
//     idx: usize,
// }

// impl<A: Ord> std::iter::Iterator for IntoIter {
//     type Item = Dot<A>;

//     fn next(&mut self) -> Option<Dot<A>> {

//         if
//         self.idx;

//         self.btree_iter
//             .next()
//             .map(|(actor, counter)| Dot::new(actor, counter))
//     }
// }

// impl<A: Ord + Default + Copy> std::iter::IntoIterator for EVClock<A> {
//     type Item = Dot<A>;
//     type IntoIter = IntoIter<A>;

//     /// Consumes the EVClock and returns an iterator over dots in the clock
//     fn into_iter(self) -> Self::IntoIter {
//         IntoIter {
//             btree_iter: self.dots.into_iter(),
//         }
//     }
// }

impl std::iter::FromIterator<Dot<usize>> for EVClock {
    fn from_iter<I: IntoIterator<Item = Dot<usize>>>(iter: I) -> Self {
        let mut clock = EVClock::default();

        for dot in iter {
            clock.apply(dot);
        }

        clock
    }
}

impl From<Dot<usize>> for EVClock {
    fn from(dot: Dot<usize>) -> Self {
        let mut clock = EVClock::default();
        clock.apply(dot);
        clock
    }
}
