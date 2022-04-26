use std::{collections::HashMap, str::FromStr};

use hydroflow::lang::{
    lattice::{dom_pair::DomPairRepr, map_union::MapUnionRepr, ord::MaxRepr},
    tag,
};
use rand::prelude::Distribution;
use zipf::ZipfDistribution;

pub(crate) type Clock = HashMap<usize, u64>;

#[derive(Debug)]
pub(crate) enum Message<K, V>
where
    K: Send + Clone,
    V: Send + Clone,
{
    // A KV set request from a client.
    Set(K, V),
    // A KV get request from a client.
    #[allow(unused)]
    Get(K, futures::channel::oneshot::Sender<(Clock, V)>),
    // A set of data that I am responsible for, sent to me by another worker.
    Batch((usize, u64), Vec<(K, V)>),
}

pub(crate) type ClockRepr = MapUnionRepr<tag::HASH_MAP, usize, MaxRepr<u64>>;
pub(crate) type ClockUpdateRepr = MapUnionRepr<tag::SINGLE, usize, MaxRepr<u64>>;

pub(crate) type ClockedDataRepr<V> = DomPairRepr<ClockRepr, MaxRepr<V>>;
pub(crate) type ClockedUpdateRepr<V> = DomPairRepr<ClockUpdateRepr, MaxRepr<V>>;

#[derive(Clone)]
pub enum Dist {
    Uniform(usize),
    Zipf(ZipfDistribution),
}

pub trait Sample<T> {
    fn sample<R: rand::Rng>(&mut self, rng: &mut R) -> T;
}

impl Sample<u64> for Dist {
    fn sample<R: rand::Rng>(&mut self, rng: &mut R) -> u64 {
        match self {
            Self::Uniform(n) => rng.gen_range(0..*n).try_into().unwrap(),
            Self::Zipf(d) => d.sample(rng).try_into().unwrap(),
        }
    }
}

impl Sample<String> for Dist {
    fn sample<R: rand::Rng>(&mut self, rng: &mut R) -> String {
        format!(
            "key{}",
            match self {
                Self::Uniform(n) => rng.gen_range(0..*n),
                Self::Zipf(d) => d.sample(rng),
            }
        )
    }
}

impl Dist {
    pub(crate) fn uniform(n: usize) -> Self {
        Self::Uniform(n)
    }

    pub(crate) fn zipf(n: usize, theta: f64) -> Self {
        Self::Zipf(ZipfDistribution::new(n, theta).unwrap())
    }
}

impl FromStr for Dist {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut contents = s.split(',');
        match contents
            .next()
            .ok_or("type of distribution is required (uniform, zipf)")?
        {
            "uniform" => {
                let n: usize = contents
                    .next()
                    .ok_or("require a number of keys parameter")?
                    .parse()
                    .map_err(|e| format!("{}", e))?;

                Ok(Dist::uniform(n))
            }
            "zipf" => {
                let n: usize = contents
                    .next()
                    .ok_or("require a number of keys parameter")?
                    .parse()
                    .map_err(|e| format!("{}", e))?;

                let theta: f64 = contents
                    .next()
                    .ok_or("require a theta parameter")?
                    .parse()
                    .map_err(|e| format!("{}", e))?;

                Ok(Dist::zipf(n, theta))
            }
            _ => Err("invalid distribution".into()),
        }
    }
}
