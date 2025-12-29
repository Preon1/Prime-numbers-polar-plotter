use std::time::Instant;

fn main() {
    // Hardcoded semiprime (product of two primes)
    // 100000007 * 100000037 = 10000004400000259
    let n: u64 = 10_000_004_400_000_259;
    
    println!("Factorizing: {}", n);
    println!("Starting factorization...\n");
    
    let start = Instant::now();
    
    let (factor1, factor2) = if n % 2 == 0 {
        // Handle even case
        (2, n >> 1)
    } else {
        // Calculate sqrt limit once
        let limit = (n as f64).sqrt() as u64 + 1;
        
        // Only check odd numbers starting from 3
        let mut divisor = 3u64;
        let mut factor1 = 0u64;
        let mut factor2 = 0u64;
        
        while divisor <= limit {
            if n % divisor == 0 {
                factor1 = divisor;
                factor2 = n / divisor;
                break;
            }
            divisor += 2;
        }
        
        if factor1 == 0 {
            println!("Number is prime!");
            return;
        }
        
        (factor1, factor2)
    };
    
    let duration = start.elapsed();
    
    println!("Factorization complete!");
    println!("Factor 1: {}", factor1);
    println!("Factor 2: {}", factor2);
    println!("Verification: {} * {} = {}", factor1, factor2, factor1 * factor2);
    println!("\nExecution time: {:.6} seconds", duration.as_secs_f64());
}
