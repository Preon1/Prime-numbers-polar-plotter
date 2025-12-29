use std::time::Instant;
use std::thread;
use std::sync::{Arc, Mutex};

fn pretty_print_int(i: u64) -> String {
    let mut s = String::new();
    let i_str = i.to_string();
    let a = i_str.chars().rev().enumerate();
    for (idx, val) in a {
        if idx != 0 && idx % 3 == 0 {
            s.insert(0, ' ');
        }
        s.insert(0, val);
    }
    return s;
}

fn is_prime(n:&u64)->bool{
	let limit = (*n as f64).sqrt() as u64 +1;
	let mut divisor = 3;

	while divisor <= limit {
		if n % divisor == 0 {
			return false;
		}
		divisor += 2;
	}
	return true;
}

fn main(){
	let args: Vec<String> = std::env::args().collect();
	let time_limit = if args.len() > 1 {
		args[1].parse::<f64>().unwrap_or(10.0)
	} else {
		10.0
	};


	let num_threads = thread::available_parallelism().unwrap().get();
	
	let results = Arc::new(Mutex::new(Vec::new()));
	
	let mut handles = vec![];
	
	for i in 0..num_threads {
		let results_clone = Arc::clone(&results);
		let step = (2 * num_threads) as u64;
		
		let handle = thread::spawn(move || {
			let mut prime_counter = 0;
			let mut last_prime = 0;
			let mut n = (3 + 2 * i) as u64;
			
			let start_time_clone = Instant::now();
			while start_time_clone.elapsed().as_secs_f64() < time_limit {
				if is_prime(&n) {
					prime_counter += 1;
					last_prime = n;
				}
				n += step;
			}
			
			let mut results = results_clone.lock().unwrap();
			results.push((prime_counter, last_prime));
		});
		
		handles.push(handle);
	}
	
	// Wait for all threads to complete
	for handle in handles {
		handle.join().unwrap();
	}
	
	// Aggregate results
	let results = results.lock().unwrap();
	let mut prime_counter = 0;
	let mut last_prime = 0;
	
	for (count, last) in results.iter() {
		prime_counter += count;
		if *last > last_prime {
			last_prime = *last;
		}
	}
	println!(
		"In {time_limit} seconds found {:>20} primes using {} threads. Biggest is {:>20}.",
		pretty_print_int(prime_counter),
		num_threads,
		pretty_print_int(last_prime),
	);
}