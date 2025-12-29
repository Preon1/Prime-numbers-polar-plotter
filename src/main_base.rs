use std::time::Instant;

fn main() {
    // Hardcoded semiprime (product of two primes)
    // 100000007 * 100000037 = 10000004400000259
    let n: u64 = 10_000_004_400_000_259;
    
    println!("Factorizing: {}", n);
    println!("Starting factorization...\n");
    
    let start = Instant::now();
    
    // Simple trial division
    let mut divisor = 2;
    let mut factor1 = 0;
    let mut factor2 = 0;
    
    loop {
        if n % divisor == 0 {
            factor1 = divisor;
            factor2 = n / divisor;
            break;
        }
        
        // Try next number
        if divisor == 2 {
            divisor = 3;
        } else {
            divisor += 2; // Skip even numbers
        }
        
        // If we've gone past sqrt(n), the number is prime
        if divisor * divisor > n {
            println!("Number is prime!");
            return;
        }
    }
    
    let duration = start.elapsed();
    
    println!("Factorization complete!");
    println!("Factor 1: {}", factor1);
    println!("Factor 2: {}", factor2);
    println!("Verification: {} * {} = {}", factor1, factor2, factor1 * factor2);
    println!("\nExecution time: {:.6} seconds", duration.as_secs_f64());
}
