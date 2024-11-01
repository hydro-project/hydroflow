pub type InputType = u128;
use std::{ops::Add, vec};
use arbitrary::{Arbitrary, Result, Unstructured}; 

pub fn default_g<T>(x: T, y: T) -> T
where
    T: Add<Output = T> + Clone,
{
    x + y
}

pub fn default_q(x: InputType) -> InputType {
    x
}
 
#[derive(Debug)]
pub struct TestingInput {
    pub i1: InputType,
    pub i2: InputType,
    pub i3: InputType,
}

impl<'a> Arbitrary<'a> for TestingInput {
    fn arbitrary(u: &mut Unstructured) -> arbitrary::Result<Self> {
        let mut i1 = InputType::arbitrary(u)?;
        let mut i2 = InputType::arbitrary(u)?;
        let mut i3 = InputType::arbitrary(u)?;

        // Ensure that at least two of the values are non-zero
        let zero_count = [i1, i2, i3].iter().filter(|&&x| x == 0).count();
        if zero_count > 1 {
            // Set two values to non-zero if more than one is zero
            if i1 == 0 { i1 = InputType::arbitrary(u)?; }
            if i2 == 0 { i2 = InputType::arbitrary(u)?; }
            if i3 == 0 { i3 = InputType::arbitrary(u)?; }
    }
    Ok(TestingInput { i1, i2, i3 })
    
}
}

