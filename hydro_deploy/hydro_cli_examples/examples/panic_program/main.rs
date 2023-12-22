use std::io::Write;

#[hydroflow::main]
async fn main() {
    let _ = hydroflow::util::cli::init::<()>().await;
    println!("hello!");

    std::io::stdout().flush().unwrap();

    panic!();
}
