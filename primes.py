import time
import math


def is_prime(n):
    # if n < 2:
    #     return False
    # if n == 2:
    #     return True
    # if n % 2 == 0:
    #     return False
    
    limit = int(math.sqrt(n)) + 1
    divisor = 3
    
    while divisor <= limit:
        if n % divisor == 0:
            return False
        divisor += 2
    
    return True


def main():
    start_time = time.time()
    
    time_limit = 10.0
    prime_counter = 0
    last_prime = 3
    
    n = 3
    while (time.time() - start_time) < time_limit:
        if is_prime(n):
            prime_counter += 1
            last_prime = n
        n += 2
    
    print(f"In {time_limit} seconds found {prime_counter} primes. Biggest is {last_prime}.")


if __name__ == "__main__":
    main()
