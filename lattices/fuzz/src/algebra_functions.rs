
pub struct FuzzFunctions<T> {
    pub f: fn(T, T) -> T,
    
    pub g: Option<fn(T, T) -> T>,
    pub q: Option<fn(T) -> T>, 
}

impl<T> FuzzFunctions<T> {
    pub fn new(
        f: fn(T, T) -> T,
        g: Option<fn(T, T) -> T>,
        q: Option<fn(T) -> T>,
    ) -> Self {
        FuzzFunctions {
            f,
            g,
            q,
        }
    }
}
