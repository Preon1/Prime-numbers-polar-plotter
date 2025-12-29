import time

def main():
    # Hardcoded semiprime (product of two primes)
    # 100000007 * 100000037 = 10000004400000259
    n = 10_000_004_400_000_259
    
    print(f"Factorizing: {n}")
    print("Starting factorization...\n")
    
    start = time.time()
    
    # Simple trial division
    divisor = 2
    factor1 = 0
    factor2 = 0
    
    while True:
        if n % divisor == 0:
            factor1 = divisor
            factor2 = n // divisor
            break
        
        # Try next number
        if divisor == 2:
            divisor = 3
        else:
            divisor += 2  # Skip even numbers
        
        # If we've gone past sqrt(n), the number is prime
        if divisor * divisor > n:
            print("Number is prime!")
            return
    
    duration = time.time() - start
    
    print("Factorization complete!")
    print(f"Factor 1: {factor1}")
    print(f"Factor 2: {factor2}")
    print(f"Verification: {factor1} * {factor2} = {factor1 * factor2}")
    print(f"\nExecution time: {duration:.6f} seconds")

if __name__ == "__main__":
    main()
