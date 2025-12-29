use std::time::Instant;


fn is_prime(n:&u64)->bool{
	let limit:u32 = (*n as f64).sqrt() as u32 +1;
	let mut divisor:u32 = 3u32;

	while divisor <= limit {
		if n % divisor == 0 {
			return false;
		}
		divisor += 2u32;
	}
	return true;
}

fn main(){
	let startTime = Instant::now();

	let timeLimit = 10f64;
	let mut primeCounter = 0u32;
	let mut lastPrime = 3u32;

	let mut n = 3u32;
	while startTime.elapsed().as_secs_f64() < timeLimit{
		if is_prime(&n){
			primeCounter += 1u32;
			lastPrime = n;
		}
		n += 2u32;
	}


	println!("In {timeLimit} seconds found {primeCounter} primes. Biggest is {lastPrime}.");
}