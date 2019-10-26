#![feature(pattern)]

#[macro_use] extern crate lazy_static;
extern crate ndarray;
extern crate regex;

use std::{cmp, env, fs, mem};
use std::io::{self, BufRead, Write};
use std::str::FromStr;
use std::collections::{BTreeSet, BTreeMap, HashMap};
use regex::Regex;

/// All extractable data from a single micro-benchmark.
#[derive(Clone, Debug)]
pub struct Benchmark {
    pub name: String,
    pub ns: u64,
    pub variance: u64,
    pub throughput: Option<u64>,
}

impl Eq for Benchmark {}

impl PartialEq for Benchmark {
    fn eq(&self, other: &Benchmark) -> bool {
        self.name == other.name
    }
}

impl Ord for Benchmark {
    fn cmp(&self, other: &Benchmark) -> cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for Benchmark {
    fn partial_cmp(&self, other: &Benchmark) -> Option<cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

lazy_static! {
    static ref BENCHMARK_REGEX: Regex = Regex::new(r##"(?x)
        test\s+(?P<name>\S+)                        # test   mod::test_name
        \s+...\sbench:\s+(?P<ns>[0-9,]+)\s+ns/iter  # ... bench: 1234 ns/iter
        \s+\(\+/-\s+(?P<variance>[0-9,]+)\)         # (+/- 4321)
        (?:\s+=\s+(?P<throughput>[0-9,]+)\sMB/s)?   # =   2314 MB/s
    "##).unwrap();
}

impl FromStr for Benchmark {
    type Err = ();

    /// Parses a single benchmark line into a Benchmark.
    fn from_str(line: &str) -> Result<Benchmark, ()> {
        let caps = match BENCHMARK_REGEX.captures(line) {
            None => return Err(()),
            Some(caps) => caps,
        };
        let ns = match parse_commas(&caps["ns"]) {
            None => return Err(()),
            Some(ns) => ns,
        };
        let variance = match parse_commas(&caps["variance"]) {
            None => return Err(()),
            Some(variance) => variance,
        };
        let throughput = caps.name("throughput").and_then(|m| parse_commas(m.as_str()));
        Ok(Benchmark {
            name: caps["name"].to_string(),
            ns: ns,
            variance: variance,
            throughput: throughput,
        })
    }
}

fn main() {
    let filename = env::args().nth(1).expect("Missing benchmarks file");
    let file = fs::File::open(filename).unwrap();
    let file = io::BufReader::new(file);

    let mut tests = HashMap::new();
    for line in file.lines() {
        let line = line.unwrap();

        if let Ok(mut bench) = Benchmark::from_str(&line) {
            bench.name = bench.name.replace("bench::", "");

            // extract: btree, vec, multi, duo...
            match extract_first_module(&mut bench.name) {
                Some(module) => {
                    // extract: difference, intersection, union...
                    match extract_first_module(&mut bench.name) {
                        Some(test) => {
                            let modules = tests.entry(test).or_insert(BTreeMap::new());
                            let map = modules.entry(module).or_insert(HashMap::new());
                            map.insert(bench.name.clone(), bench);
                        },
                        None => eprintln!("No test found for bench {:?}", bench.name),
                    }
                },
                None => eprintln!("No module found for bench {:?}", bench.name),
            }
        }
    }

    // generate the arrays of the form
    //
    // difference.data
    //
    // Title               btree   multi   duo
    // "two slices big"      799     821   498
    // "two slices big2"    1152     474   288
    // "two slices big3"    1223     108    75
    // "three slices big"    932    1210   NaN
    // "three slices big2"  2954     819   NaN
    // "three slices big3"  7191     111   NaN

    for (test, modules) in &tests {
        let filename = format!("{}.data", test);
        let mut file = fs::File::create(filename).unwrap();

        // `+ 1` for titles and benchmark names
        let row = modules.iter().map(|(_, b)| b.len()).max().unwrap_or(0) + 1;
        let col = modules.len() + 1;
        let mut array = vec![vec![String::new(); col]; row];

        // write titles
        array[0][0].push_str("Title");
        for (x, (title, _)) in array[0].iter_mut().skip(1).zip(modules.iter()) {
            x.push_str(title);
        }

        let mut benchmarks = BTreeSet::new();
        for (_, benches) in modules {
            for (_, bench) in benches {
                benchmarks.insert(bench.name.clone());
            }
        }

        // write benchmarks names
        for (row, benchname) in array.iter_mut().skip(1).zip(benchmarks.iter()) {
            let name = format!("{:?}", benchname.replace("_", " "));
            row[0].push_str(&name);
            for (tile, (_, benches)) in row.iter_mut().skip(1).zip(modules.iter()) {
                match benches.get(benchname) {
                    Some(bench) => tile.push_str(&bench.ns.to_string()),
                    None => tile.push_str("NaN"),
                }
            }
        }

        // write to file
        let mut aligns = vec![0; col];
        for i in 0..col {
            if let Some(align) = array.iter().map(|v| v[i].len()).max() {
                aligns[i] = align;
            }
        }

        for row in array {
            for (i, tile) in row.iter().enumerate() {
                write!(&mut file, "{1:0$} ", aligns[i] + 3, tile).unwrap();
            }
            writeln!(&mut file).unwrap();
        }
    }
}

fn extract_first_module(s: &mut String) -> Option<String> {
    match s.find("::") {
        Some(i) => {
            let mut name = s.split_off(i + 2);
            mem::swap(s, &mut name);

            let new_len = name.len() - 2;
            name.truncate(new_len);
            Some(name)
        },
        None => None,
    }
}

/// Drops all commas in a string and parses it as a unsigned integer
fn parse_commas(s: &str) -> Option<u64> {
    drop_commas(s).parse().ok()
}

/// Drops all commas in a string
fn drop_commas(s: &str) -> String {
    s.chars().filter(|&b| b != ',').collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_first_module_easy() {
        let mut name = "vec::kiki::koko".to_string();
        let module = extract_first_module(&mut name);
        assert_eq!(Some("vec".into()), module);
        assert_eq!("kiki::koko".to_string(), name);
    }

    #[test]
    fn extract_first_module_no_module() {
        let mut name = "koko".to_string();
        let module = extract_first_module(&mut name);
        assert_eq!(None, module);
        assert_eq!("koko".to_string(), name);
    }
}
