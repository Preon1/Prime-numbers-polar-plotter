use std::time::Instant;
use std::thread;
use std::sync::{Arc, Mutex};
use image::{ImageBuffer, Rgb};

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

	let image_size = if args.len() > 2 {
		args[2].parse::<u32>().unwrap_or(1000)
	} else {
		1000
	};

	let max_radius = if args.len() > 3 {
		args[3].parse::<f64>().unwrap_or(100000.0)
	} else {
		100000.0
	};
	let draw_radius = max_radius * f64::sqrt(2.0);
	
	let pixel_grow = if args.len() > 4 {
		args[4].parse::<f64>().unwrap_or(5.0)
	} else {
		5.0
	};

	let threads = if args.len() > 5 {
		args[5].parse::<usize>().unwrap_or(0)
	} else {
		0
	};

	// 2+ - colored, 1 - paired match, 0 - white-only
	let colored = if args.len() > 6 {
		args[6].parse::<i8>().unwrap_or(0)
	} else {
		0
	};



	let num_threads = if threads == 0 {
		thread::available_parallelism().unwrap().get()
	} else {
		threads
	};

	println!("Starting max {time_limit}s run with {num_threads} threads...");

	let start_time = Instant::now();
	
	let results = Arc::new(Mutex::new(Vec::new()));
	
	let mut handles = vec![];
	
	for i in 0..num_threads {
		let results_clone = Arc::clone(&results);
		let step = (2 * num_threads) as u64;
		
		let handle = thread::spawn(move || {
			let mut primes = Vec::new();
			let mut n = (3 + 2 * i) as u64;
			
			let start_time_clone = Instant::now();
			while start_time_clone.elapsed().as_secs_f64() < time_limit {
				if is_prime(&n) {
					primes.push(n);
				}

				n += step;

				if n as f64 > draw_radius{ break; }
			}
			
			let mut results = results_clone.lock().unwrap();
			results.extend(primes);
		});
		
		handles.push(handle);
	}
	
	// Wait for all threads to complete
	for handle in handles {
		handle.join().unwrap();
	}
	
	// Aggregate results
	let mut results = results.lock().unwrap();
	let prime_counter = results.len() as u64;
	let last_prime = *results.iter().max().unwrap_or(&0);
	
	println!(
		"Found {} primes in {}s. Biggest is {}.",
		pretty_print_int(prime_counter),
		start_time.elapsed().as_secs_f64(),
		pretty_print_int(last_prime),
	);



	// if to display colored pairs - presort results vector
	if colored == 1 {
		println!("Sorting...");
		results.sort();
	}
	let results_length = results.len();


	// Generate polar plot
	println!("Generating polar plot image {}px with max radius {}...", image_size, pretty_print_int(max_radius as u64));
	
	let mut img = ImageBuffer::from_pixel(image_size, image_size, Rgb([0u8, 0u8, 0u8]));
	let center = image_size as f64 / 2.0;
	let scale = (image_size as f64 / 2.0) / max_radius;
	let mut drawn = 0u64;

	for &prime in results.iter() {
        if prime as f64 > draw_radius {
            continue;
        }
        
        let angle = prime as f64;
        let radius = prime as f64 * scale;
        
        let x = center + radius * angle.cos();
        let y = center + radius * angle.sin();
        
        if 
        x >= 0.0
        && x < image_size as f64
        && y >= 0.0
        && y < image_size as f64
        {
			drawn += 1u64;

            let px = x as i32;
            let py = y as i32;

			
			let pixel = if colored > 1 {
				match (prime % 10) as u8 {
					1 => (0u8, 255u8, 255u8),
					3 => (255u8, 0u8, 255u8),
					5 => (255u8, 255u8, 0u8),
					7 => (0u8, 255u8, 0u8),
					9 => (0u8, 0u8, 255u8),
					_ => (255u8, 0u8, 0u8)
				}
			} else if colored == 1 {
				//colored neighbors - using sorted vector
				
				// Binary search to find current prime's position
				let pos = results.binary_search(&prime).unwrap();
				
				let trailing = pos > 0 && results[pos - 1] == prime - 2;
				let leading = pos < results_length - 1 && results[pos + 1] == prime + 2;

				match (trailing, leading) {
					(true, true) => (0u8, 0u8, 255u8),   // both neighbors - blue
					(true, false) => (0u8, 255u8, 0u8),  // trailing only - green
					(false, true) => (255u8, 0u8, 0u8),  // leading only - red
					_ => (50u8, 50u8, 50u8)           // no neighbors - grey
				}

			} else {
				(255u8, 255u8, 255u8)
			};

			if pixel_grow == 1.0 {
				img.put_pixel(px as u32, py as u32, Rgb([pixel.0, pixel.1, pixel.2]));
				continue;
			}
            
            // Calculate point size based on distance from center
            let distance_ratio = radius / (image_size as f64 / 2.0);
            let point_radius = distance_ratio * pixel_grow;
            
            // Draw circular point with gradient
            let r_int = point_radius.ceil() as i32;
            
            for dx in -r_int..=r_int {
                for dy in -r_int..=r_int {
                    let dist_from_point = ((dx * dx + dy * dy) as f64).sqrt();
                    
                    if dist_from_point <= point_radius {
                        // Calculate intensity: 1.0 at center, fades to 0.0 at edge

                        let intensity = 1.0 - (dist_from_point / point_radius);
                        let brightness = (
							(intensity * pixel.0 as f64) as u8,
							(intensity * pixel.1 as f64) as u8,
							(intensity * pixel.2 as f64) as u8,
						);
                        
                        let nx = px + dx;
                        let ny = py + dy;
                        if nx >= 0 && nx < image_size as i32 && ny >= 0 && ny < image_size as i32 {
                            let current = img.get_pixel(nx as u32, ny as u32);
							let new_brightness = (
								current[0].max(brightness.0),
								current[1].max(brightness.1),
								current[2].max(brightness.2),
							);

							img.put_pixel(nx as u32, ny as u32, Rgb([new_brightness.0, new_brightness.1, new_brightness.2]));
                        }
                    }
                }
            }
            
        }
    }
	
	let filename = format!("{}K_primes_{}_rad_{}_grow_{}_color_{}.png",
	image_size/1000, drawn, max_radius, pixel_grow, colored);
	img.save(&filename).expect("Failed to save image");
	println!("Saved polar plot to {}", filename);
}