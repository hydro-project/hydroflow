#[cfg(not(feature = "macro"))]
stageleft::stageleft_crate!(stageleft_test_macro);

use stageleft::{q, IntoQuotedOnce, Quoted, RuntimeData};

#[stageleft::entry]
pub fn raise_to_power(ctx: &(), value: RuntimeData<i32>, power: u32) -> impl Quoted<i32> {
    if power == 1 {
        q!(value).boxed()
    } else if power % 2 == 0 {
        let half_result = raise_to_power(ctx, value, power / 2);
        q!({
            let v = half_result;
            v * v
        })
        .boxed()
    } else {
        let half_result = raise_to_power(ctx, value, power / 2);
        q!({
            let v = half_result;
            (v * v) * value
        })
        .boxed()
    }
}

#[stageleft::runtime]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raise_to_power_of_two() {
        let result = raise_to_power!(2.into(), 10);
        assert_eq!(result, 1024);
    }

    #[test]
    fn test_raise_to_odd_power() {
        let result = raise_to_power!(2.into(), 5);
        assert_eq!(result, 32);
    }
}
