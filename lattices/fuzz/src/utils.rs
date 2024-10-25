use std::{ops::Add};
use arbitrary::{Arbitrary, Result, Unstructured}; 

pub type InputType = u8; 

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
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let i1 = InputType::arbitrary(u)?;
        let i2 = InputType::arbitrary(u)?;
        let i3 = InputType::arbitrary(u)?;
        Ok(TestingInput { i1, i2, i3 })
    }
}
