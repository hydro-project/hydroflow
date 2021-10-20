pub struct Token<const C: char>;

pub trait Order {
    // TODO!
}

pub struct EmptyOrder;
impl Order for EmptyOrder {}

#[test]
fn test_token() {
    let _ = Token::<'明'>;
}
