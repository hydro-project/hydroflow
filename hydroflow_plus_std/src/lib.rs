stageleft::stageleft_no_entry_crate!();

pub mod quorum;
pub mod request_response;

#[stageleft::runtime]
#[cfg(test)]
mod tests {
    #[ctor::ctor]
    fn init() {
        hydroflow_plus::deploy::init_test();
    }
}
