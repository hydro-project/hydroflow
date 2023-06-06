use multiplatform_test::multiplatform_test;

#[multiplatform_test]
fn test_default() {}

#[multiplatform_test(test, env_tracing)]
fn test_tracing() {
    tracing::warn!("This is a tracing warning!");
}

#[multiplatform_test(test, env_logging)]
fn test_logging() {
    log::warn!("This is a logging warning!");
}
