use core::cmp::Ordering;
use core::convert::Infallible;
use core::fmt::{self, Debug, Display};

use serde::{Deserialize, Serialize};

use crate::efficient_vclock::EVClock;
use crdts::{CmRDT, CvRDT, Dot};

/// ReadCtx's are used to extract data from CRDT's while maintaining some causal history.
/// You should store ReadCtx's close to where mutation is exposed to the user.
///
/// e.g. Ship ReadCtx to the clients, then derive an Add/RmCtx and ship that back to
/// where the CRDT is stored to perform the mutation operation.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EReadCtx<V> {
    /// clock used to derive an AddCtx
    pub clock: EVClock,

    /// the data read from the CRDT
    pub val: V,
}

/// AddCtx is used for mutations add new information to a CRDT
#[derive(Debug, Serialize, Deserialize)]
pub struct EAddCtx {
    /// The adding vclock context
    pub clock: EVClock,

    /// The Actor and the Actor's version at the time of the add
    pub dot: Dot<usize>,
}

/// MVReg (Multi-Value Register)
/// On concurrent writes, we will keep all values for which
/// we can't establish a causal history.
///
/// ```rust
/// use crdts::{CmRDT, MVReg, Dot, VClock};
/// let mut r1 = MVReg::new();
/// let mut r2 = r1.clone();
/// let r1_read_ctx = r1.read();
/// let r2_read_ctx = r2.read();
///
/// r1.apply(r1.write("bob", r1_read_ctx.derive_add_ctx(123)));
///
/// let op = r2.write("alice", r2_read_ctx.derive_add_ctx(111));
/// r2.apply(op.clone());
///
/// r1.apply(op); // we replicate op to r1
///
/// // Since "bob" and "alice" were added concurrently, we see both on read
/// assert_eq!(r1.read().val, vec!["bob", "alice"]);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EMaxReg<V> {
    clock: EVClock,
    val: V,
}

/// Defines the set of operations over the MVReg
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Op<V> {
    /// Put a value
    Put {
        /// context of the operation
        clock: EVClock,
        /// the value to put
        val: V,
    },
}

impl<V: Display> Display for EMaxReg<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "|")?;
        write!(f, "{}@{}", self.val, self.clock)?;
        write!(f, "|")
    }
}

impl<V: PartialEq> PartialEq for EMaxReg<V> {
    fn eq(&self, other: &Self) -> bool {
        for dot in self.clock.iter() {
            let num_found = other.clock.iter().filter(|d| d == &dot).count();

            if num_found == 0 {
                return false;
            }
            // sanity check
            assert_eq!(num_found, 1);
        }
        for dot in other.clock.iter() {
            let num_found = self.clock.iter().filter(|d| d == &dot).count();

            if num_found == 0 {
                return false;
            }
            // sanity check
            assert_eq!(num_found, 1);
        }
        true
    }
}

impl<V: Eq> Eq for EMaxReg<V> {}

// impl<V> EResetRemove for EMaxReg<V> {
//     fn reset_remove(&mut self, clock: &EVClock) {
//         self.vals = mem::take(&mut self.vals)
//             .into_iter()
//             .filter_map(|(mut val_clock, val)| {
//                 val_clock.reset_remove(clock);
//                 if val_clock.is_empty() {
//                     None // remove this value from the register
//                 } else {
//                     Some((val_clock, val))
//                 }
//             })
//             .collect()
//     }
// }

impl<V: Default> Default for EMaxReg<V> {
    fn default() -> Self {
        Self {
            clock: EVClock::default(),
            val: V::default(),
        }
    }
}

impl<V: Ord> CvRDT for EMaxReg<V> {
    type Validation = Infallible;

    fn validate_merge(&self, _other: &Self) -> Result<(), Self::Validation> {
        Ok(())
    }

    fn merge(&mut self, other: Self) {
        match self.clock.partial_cmp(&other.clock) {
            Some(ord) => match ord {
                Ordering::Less => {
                    self.val = other.val;
                    self.clock = other.clock;
                }
                Ordering::Equal => {}
                Ordering::Greater => {}
            },
            None => {
                // Concurrent edit means take max.
                if other.val > self.val {
                    self.val = other.val;
                }
            }
        }
    }
}

impl<V: Ord> CmRDT for EMaxReg<V> {
    type Op = Op<V>;
    type Validation = Infallible;

    fn validate_op(&self, _op: &Self::Op) -> Result<(), Self::Validation> {
        Ok(())
    }

    fn apply(&mut self, op: Self::Op) {
        match op {
            Op::Put { clock, val } => {
                self.merge(EMaxReg { clock, val });
            }
        }
    }
}

impl<V> EMaxReg<V> {
    /// Construct a new empty MVReg
    pub fn new(val: V) -> Self {
        Self {
            clock: EVClock::default(),
            val,
        }
    }

    /// Set the value of the register
    pub fn write(&self, val: V, ctx: EAddCtx) -> Op<V> {
        Op::Put {
            clock: ctx.clock,
            val,
        }
    }

    /// Consumes the register and returns the values
    pub fn read(&self) -> EReadCtx<V>
    where
        V: Clone,
    {
        EReadCtx {
            clock: self.clock.clone(),
            val: self.val.clone(),
        }
    }

    /// Retrieve the current read context
    pub fn read_ctx(&self) -> EReadCtx<()> {
        EReadCtx {
            clock: self.clock.clone(),
            val: (),
        }
    }
}
