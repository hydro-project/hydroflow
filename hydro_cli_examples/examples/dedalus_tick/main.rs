use hydroflow::tokio_stream::wrappers::IntervalStream;
use hydroflow_datalog::datalog;
use tokio::time::{interval_at, Duration, Instant};

#[tokio::main]
async fn main() {
    let _ = hydroflow::util::cli::init().await;

    let frequency = 1;
    let start = Instant::now() + Duration::from_secs(frequency);
    let periodic_source = IntervalStream::new(interval_at(start, Duration::from_secs(frequency)));

    let mut df = datalog!(
        r#"
        .input clientIn `repeat_iter(vec![()]) -> map(|_| (context.current_tick(),))`
        .input periodic `source_stream(periodic_source) -> map(|_| () )`
        .output throughputOut `for_each(|(num,):(u32,)| println!("committed {:?} entries", num))`
        .input startSlot `repeat_iter([(0 as u32,),])`

        nextSlot(s) :+ startSlot(s), !nextSlot(s2)
        nextSlot(s+1) :+ nextSlot(s)
        throughputOut(s) :- nextSlot(s), clientIn(rand), periodic()
        "#
    );

    df.run_async().await;
}