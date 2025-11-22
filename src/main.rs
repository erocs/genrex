use std::{env, process};
use rand::rngs::StdRng;
use rand::SeedableRng;
use genrex::{RegexGeneratorBuilder, GeneratorConfig};
use std::time::Duration;

fn print_usage() {
    eprintln!("Usage: genrex-cli <pattern> [--n N] [--seed S] [--min M] [--max M] [--attempts A] [--timeout-ms T] [--multiline] [--allow-backrefs] [-v]");
}

fn main() {
    let mut args = env::args().skip(1);
    let pattern = match args.next() {
        Some(p) => p,
        None => {
            print_usage();
            process::exit(2);
        }
    };

    // Defaults
    let mut n: usize = 1;
    let mut seed: Option<u64> = None;
    let mut min_len: Option<usize> = None;
    let mut max_len: Option<usize> = None;
    let mut max_attempts: Option<usize> = None;
    let mut timeout_ms: Option<u64> = None;
    let mut multiline = false;
    let mut allow_backrefs = false;
    let mut verbose = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--n" => {
                if let Some(v) = args.next() { n = v.parse().unwrap_or(1); }
            }
            "--seed" => {
                if let Some(v) = args.next() { seed = v.parse().ok(); }
            }
            "--min" => {
                if let Some(v) = args.next() { min_len = v.parse().ok(); }
            }
            "--max" => {
                if let Some(v) = args.next() { max_len = v.parse().ok(); }
            }
            "--attempts" => {
                if let Some(v) = args.next() { max_attempts = v.parse().ok(); }
            }
            "--timeout-ms" => {
                if let Some(v) = args.next() { timeout_ms = v.parse().ok(); }
            }
            "--multiline" => {
                multiline = true;
            }
            "--allow-backrefs" => {
                allow_backrefs = true;
            }
            "-v" => {
                verbose = true;
            }
            _ => {
                eprintln!("Unknown arg: {}", arg);
                print_usage();
                process::exit(2);
            }
        }
    }

    let mut builder = RegexGeneratorBuilder::new(&pattern);
    if let Some(min) = min_len {
        builder = builder.config(GeneratorConfig {
            min_len: min,
            max_len: max_len.unwrap_or(64),
            max_attempts: max_attempts.unwrap_or(10_000),
            timeout: timeout_ms.map(Duration::from_millis),
        });
    } else if max_len.is_some() || max_attempts.is_some() || timeout_ms.is_some() {
        builder = builder.config(GeneratorConfig {
            min_len: min_len.unwrap_or(0),
            max_len: max_len.unwrap_or(64),
            max_attempts: max_attempts.unwrap_or(10_000),
            timeout: timeout_ms.map(Duration::from_millis),
        });
    }

    if multiline {
        builder = builder.multiline(true);
    }

    if let Some(s) = seed {
        builder = builder.rng(StdRng::seed_from_u64(s));
    }
    
    if allow_backrefs {
        builder = builder.allow_backrefs();
    }
    
    if verbose {
        genrex::set_verbose(true);
    }
    
    let mut generator = match builder.build() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Failed to build generator: {:?}", e);
            process::exit(1);
        }
    };
 
    for _ in 0..n {
        match generator.generate_one() {
            Ok(s) => println!("{}", s),
            Err(e) => {
                eprintln!("Generation error: {:?}", e);
                process::exit(1);
            }
        }
    }
}