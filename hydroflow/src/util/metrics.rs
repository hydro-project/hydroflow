/// prometheus api only allows &str to be used for countervecs, so, need a way of mapping from usize to &str without allocating.
pub const STRATUM_NUM_MAP: [&str; 21] = [
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16",
    "17", "18", "19", "20",
];

/// allocate 10 buckets per order of magnitude. This is helpful for when measuring execution latencies because they can span a large dynamic range.
pub fn gen_buckets() -> Vec<f64> {
    let nano_buckets = (0..10).map(|x| f64::powf(2.0, x as f64) * 0.000000001);
    let micro_buckets = (0..10).map(|x| f64::powf(2.0, x as f64) * 0.000001);
    let milli_buckets = (0..10).map(|x| f64::powf(2.0, x as f64) * 0.001);
    let buckets = (0..10).map(|x| f64::powf(2.0, x as f64) * 1.0);

    let mut ret = Vec::new();
    ret.extend(nano_buckets);
    ret.extend(micro_buckets);
    ret.extend(milli_buckets);
    ret.extend(buckets);

    ret
}

pub struct Counters {
    pub run_stratum_latency: prometheus::HistogramVec,
    pub ticks: prometheus::Counter,
    pub next_stratum_latency: prometheus::Histogram,
    pub run_available_async_latency: prometheus::Histogram,
    pub run_available_async_yield_latency: prometheus::Histogram,
    pub recv_events_async: prometheus::Histogram,
}

impl Counters {
    pub fn new() -> Self {
        #[rustfmt::skip]
        return Self { // return because rustfmt::skip on expression is experimental.
            run_stratum_latency: prometheus::register_histogram_vec!("hydroflow_run_stratum_latency", "help", &["stratum"], gen_buckets()).unwrap(),
            ticks: prometheus::register_counter!("hydroflow_ticks", "help").unwrap(),
            next_stratum_latency: prometheus::register_histogram!("hydroflow_next_stratum_latency", "help", gen_buckets()).unwrap(),
            run_available_async_latency: prometheus::register_histogram!("hydroflow_run_available_async_latency", "help", gen_buckets()).unwrap(),
            run_available_async_yield_latency: prometheus::register_histogram!("hydroflow_run_available_async_yield_latency", "help", gen_buckets()).unwrap(),
            recv_events_async: prometheus::register_histogram!("hydroflow_recv_events_async", "help", gen_buckets()).unwrap(),
        };
    }
}
