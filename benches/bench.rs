/// e.g.
// cargo bench "cetane-u64-digits-1-1" -- --exact
use criterion::criterion_main;
use criterion::{Criterion, criterion_group};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::hint::black_box;
use std::ops::RangeInclusive;

use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

pub fn random_number(rng: &mut impl Rng, digit_range: RangeInclusive<u32>, max: u128) -> String {
    let digits = rng.gen_range(digit_range);
    let max_num = std::cmp::min(max, 10u128.pow(digits));
    let mut min_num = 10u128.pow(digits - 1);
    if digits == 1 {
        min_num = 0;
    }
    let num = rng.gen_range(min_num..max_num);
    let mut buffer = itoa::Buffer::new();
    let s = buffer.format(num);
    s.to_owned()
}

pub fn random_numbers(num: usize, digit_range: RangeInclusive<u32>, max: u128) -> Vec<String> {
    let mut rng = StdRng::seed_from_u64(42);
    std::iter::repeat(())
        .take(num)
        .map(|_| random_number(&mut rng, digit_range.clone(), max))
        .collect::<Vec<String>>()
}

pub fn write_numbers(path: &str, nums: Vec<String>) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    for s in nums {
        writeln!(writer, "{s}")?;
    }
    Ok(())
}

pub fn read_numbers(path: &str) -> std::io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    reader.lines().collect()
}

pub fn benchmark_parser<F, T>(c: &mut Criterion, name: &str, parser: F, vals: &[String])
where
    F: Fn(&str) -> Option<T> + 'static,
{
    c.bench_function(name, |b| {
        b.iter(|| {
            for val in vals.iter() {
                let _ = black_box(parser(&val));
            }
        })
    });
}

macro_rules! bench_integer_type {
    ($c:ident, $type:ty) => {{
        let abs_max_digits = <$type>::MAX.to_string().len() as u32;
        for max_digit in 1..=abs_max_digits {
            let mut min_digits = vec![1];
            if max_digit > 1 {
                min_digits.push(max_digit);
            }

            for min_digit in min_digits {
                // Write then read numbers to disk to black box compiler
                let vals = random_numbers(1000, min_digit..=max_digit, <$type>::MAX as u128);
                write_numbers("data/data.txt", vals).unwrap();
                let vals = read_numbers("data/data.txt").unwrap();

                benchmark_parser(
                    $c,
                    &format!(
                        "std-{}-digits-{}-to-{}",
                        stringify!($type),
                        min_digit,
                        max_digit
                    ),
                    |s| s.parse::<$type>().ok(),
                    &vals,
                );

                benchmark_parser(
                    $c,
                    &format!(
                        "lexical-core-{}-digits-{}-to-{}",
                        stringify!($type),
                        min_digit,
                        max_digit
                    ),
                    |s| lexical_core::parse::<$type>(s.as_bytes()).ok(),
                    &vals,
                );

                benchmark_parser(
                    $c,
                    &format!(
                        "atoi-{}-digits-{}-to-{}",
                        stringify!($type),
                        min_digit,
                        max_digit
                    ),
                    |s| atoi::atoi::<$type>(s.as_bytes()),
                    &vals,
                );

                benchmark_parser(
                    $c,
                    &format!(
                        "atoi_simd-{}-digits-{}-to-{}",
                        stringify!($type),
                        min_digit,
                        max_digit
                    ),
                    |s| atoi_simd::parse::<$type>(s.as_bytes()).ok(),
                    &vals,
                );

                benchmark_parser(
                    $c,
                    &format!(
                        "cetane-{}-digits-{}-to-{}",
                        stringify!($type),
                        min_digit,
                        max_digit
                    ),
                    |s| cetane::atoi::<$type>(s.as_bytes()).ok(),
                    &vals,
                );
            }
        }
    }};
}

fn benchmark(c: &mut Criterion) {
    bench_integer_type!(c, u64);
}

criterion_group!(bench, benchmark);
criterion_main!(bench);
