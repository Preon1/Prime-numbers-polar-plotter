import time
import math
from multiprocessing import Process, Queue, cpu_count


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


def worker(start, step, time_limit, start_time, result_queue):
    """Worker process that checks primes starting from 'start' with given 'step'"""
    prime_counter = 0
    last_prime = 0
    n = start
    
    while (time.time() - start_time) < time_limit:
        if is_prime(n):
            prime_counter += 1
            last_prime = n
        n += step
    
    result_queue.put((prime_counter, last_prime))


def main():
    start_time = time.time()
    
    time_limit = 10.0
    num_workers = cpu_count()
    
    # Create a queue to collect results
    result_queue = Queue()
    
    # Start worker processes
    processes = []
    for i in range(num_workers):
        # Each worker starts at (3 + 2*i) and steps by (2 * num_workers)
        # This ensures no overlap and all odd numbers are covered
        p = Process(target=worker, args=(3 + 2*i, 2*num_workers, time_limit, start_time, result_queue))
        p.start()
        processes.append(p)
    
    # Wait for all processes to complete
    for p in processes:
        p.join()
    
    # Collect results
    total_primes = 0
    biggest_prime = 0
    for _ in range(num_workers):
        prime_count, last_prime = result_queue.get()
        total_primes += prime_count
        if last_prime > biggest_prime:
            biggest_prime = last_prime
    
    print(f"In {time_limit} seconds found {total_primes} primes using {num_workers} workers. Biggest is {biggest_prime}.")


if __name__ == "__main__":
    main()
