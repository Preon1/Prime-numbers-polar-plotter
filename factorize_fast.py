import time

def main():
    # Hardcoded semiprime (product of two primes)
    # 100000007 * 100000037 = 10000004400000259
    n = 10_000_004_400_000_259
    
    print(f"Factorizing: {n}")
    print("Starting factorization...\n")
    
    start = time.perf_counter()
    
    # Optimized trial division
    # Check 2 first
    if n % 2 == 0:
        factor1 = 2
        factor2 = n >> 1  # Division by 2 using bit shift
    else:
        # Calculate sqrt limit once
        limit = int(n ** 0.5) + 1
        
        # Only check odd numbers starting from 3
        divisor = 3
        while divisor <= limit:
            if n % divisor == 0:
                factor1 = divisor
                factor2 = n // divisor
                break
            divisor += 2
        else:
            print("Number is prime!")
            return
    
    duration = time.perf_counter() - start
    
    print("Factorization complete!")
    print(f"Factor 1: {factor1}")
    print(f"Factor 2: {factor2}")
    print(f"Verification: {factor1} * {factor2} = {factor1 * factor2}")
    print(f"\nExecution time: {duration:.6f} seconds")

if __name__ == "__main__":
    main()
