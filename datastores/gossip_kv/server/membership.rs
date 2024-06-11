use std::sync::OnceLock;

use gossip_protocol::membership::MemberId;
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

/// Gets a name for the current process.
pub fn member_name() -> &'static MemberId {
    static MEMBER_NAME: OnceLock<MemberId> = OnceLock::new();
    MEMBER_NAME.get_or_init(|| {
        // Generate a lower-case alphanumeric suffix of length 4
        let suffix: String = thread_rng()
            .sample_iter(&LowercaseAlphanumeric)
            .take(4)
            .map(char::from)
            .collect();

        // Retrieve hostname
        let hostname = hostname::get().unwrap().to_str().unwrap().to_string();

        format!("{}-{}", hostname, suffix)
    })
}
