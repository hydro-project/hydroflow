use std::sync::Mutex;

use once_cell::sync::Lazy;
use rand::distributions::Distribution;
use rand::{thread_rng, Rng};

/// This is a simple distribution that generates a random lower-case alphanumeric
struct LowercaseAlphanumeric;

impl Distribution<char> for LowercaseAlphanumeric {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        let choices = b"abcdefghijklmnopqrstuvwxyz0123456789";
        choices[rng.gen_range(0..choices.len())] as char
    }
}

/// Holds a name for the current process.
static MEMBER_NAME: Lazy<Mutex<String>> = Lazy::new(|| {
    // Generate a lower-case alphanumeric suffix of length 4
    let suffix: String = thread_rng()
        .sample_iter(&LowercaseAlphanumeric)
        .take(4)
        .map(char::from)
        .collect();

    // Retrieve hostname
    let hostname = hostname::get().unwrap().to_str().unwrap().to_string();

    Mutex::new(format!("{}-{}", hostname, suffix))
});

/// Gets a name for the current process.
pub fn member_name() -> String {
    MEMBER_NAME.lock().unwrap().clone()
}
