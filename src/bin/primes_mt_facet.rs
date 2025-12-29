use std::time::Instant;
use std::thread;
use std::sync::{Arc, Mutex};


fn is_prime(n: &u64, primes: &Vec<u64>) -> bool {
    let limit = (*n as f64).sqrt() as u64 + 1;

    for &prime in primes.iter() {
        if prime > limit {
            break;
        }
        if n % prime == 0 {
            return false;
        }
    }
    true
}

fn main(){
    let start_time = Instant::now();
    
    let time_limit = 10.0;
    let num_threads = thread::available_parallelism().unwrap().get();
    
    let primes = Arc::new(Mutex::new(vec![2u64, 3u64])); // Initialize with first primes
    let start_time_shared = Arc::new(start_time);
    
    let mut handles = vec![];
    
    for i in 0..num_threads {
        let start_time_clone = Arc::clone(&start_time_shared);
        let primes_clone = Arc::clone(&primes); // Clone Arc for each thread
        let step = (2 * num_threads) as u64;
        
        let handle = thread::spawn(move || {
            let mut n = (5 + 2 * i) as u64; // Start from 5 since 2 and 3 are already in primes
            
            while start_time_clone.elapsed().as_secs_f64() < time_limit {
                let mut primes_guard = primes_clone.lock().unwrap();
                if is_prime(&n, &primes_guard) {
                    primes_guard.push(n);
                }
                drop(primes_guard); // Release lock immediately
                n += step;
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Aggregate results
    let primes_final = primes.lock().unwrap();
    let prime_counter = primes_final.len();
    let last_prime = primes_final[prime_counter - 1];


    println!("In {time_limit} seconds found {prime_counter} primes using {num_threads} threads. Biggest is {last_prime}.");
}