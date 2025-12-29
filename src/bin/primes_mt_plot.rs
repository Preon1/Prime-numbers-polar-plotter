use std::time::Instant;
use std::thread;
use std::sync::{Arc, Mutex};
use image::{ImageBuffer, Rgb};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "primes_mt_plot")]
#[command(about = "Multi-threaded prime number polar plot generator", long_about = None)]
struct Args {
    /// Time limit in seconds for prime generation
    #[arg(short = 'l', long, default_value_t = 600.0)]
    time_limit: f64,

    /// Image size in pixels (width and height)
    #[arg(short = 's', long, default_value_t = 1000)]
    image_size: u32,

    /// Maximum radius for the polar plot
    #[arg(short = 'r', long, default_value_t = 100000.0)]
    max_radius: f64,

    /// Pixel growth factor based on distance
    #[arg(short = 'g', long, default_value_t = 5.0)]
    pixel_grow: f64,

    /// Number of threads (0 = auto-detect)
    #[arg(short = 't', long, default_value_t = 0)]
    threads: usize,

    /// Coloring mode: 0=white, 1=paired neighbors, 2+=by last digit
    #[arg(short = 'c', long, default_value_t = 0)]
    colored: i8,

    /// Center bias X coordinate
    #[arg(short = 'x', long, default_value_t = 0.0)]
    center_bias_x: f64,

    /// Center bias Y coordinate
    #[arg(short = 'y', long, default_value_t = 0.0)]
    center_bias_y: f64,

    /// Fixed pixel size (overrides pixel_grow when != 1.0)
    #[arg(short = 'f', long, default_value_t = 1.0)]
    pixel_fixed_size: f64,
}

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
	let args = Args::parse();

	let time_limit = args.time_limit;
	let image_size = args.image_size;
	let max_radius = args.max_radius;
	let pixel_grow = args.pixel_grow;
	let threads = args.threads;
	let colored = args.colored;
	let center_bias_x = args.center_bias_x;
	let center_bias_y = args.center_bias_y;
	let pixel_fixed_size = args.pixel_fixed_size;

	let scale = (image_size as f64 / 2.0) / max_radius;
	let draw_radius = max_radius * f64::sqrt(2.0) + ( center_bias_x.abs().max(center_bias_y.abs()) / scale );





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

	let center_x = image_size as f64 / 2.0 + center_bias_x;
	let center_y = image_size as f64 / 2.0 + center_bias_y;
	let mut drawn = 0u64;

	for &prime in results.iter() {
        if prime as f64 > draw_radius {
            continue;
        }
        
        let angle = prime as f64;
        let radius = prime as f64 * scale;
        
        let x = center_x + radius * angle.cos();
        let y = center_y + radius * angle.sin();
        
        if 
        x >= center_bias_x * scale
        && x < image_size as f64 + center_bias_x * scale
        && y >= center_bias_y * scale
        && y < image_size as f64 + center_bias_y * scale
        {
			drawn += 1u64;

            let px = (x - center_bias_x * scale) as i32;
            let py = (y - center_bias_y * scale) as i32;

			
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

			if pixel_grow == 1.0 && pixel_fixed_size == 1.0 {
				img.put_pixel(px as u32, py as u32, Rgb([pixel.0, pixel.1, pixel.2]));
				continue;
			}
            
            // Calculate point size based on distance from center
            let distance_ratio = radius / (image_size as f64 / 2.0);
            let point_radius = if pixel_fixed_size == 1.0 {
				distance_ratio * pixel_grow
			}else{
				pixel_fixed_size
			};
            
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
	
	let filename = format!("{}K_primes_{}_rad_{}_grow_{}_color_{}_x_{}_y_{}.png",
	image_size/1000, drawn, max_radius, pixel_grow, colored, center_bias_x, center_bias_y);
	img.save(&filename).expect("Failed to save image");
	println!("Saved polar plot to {}", filename);
}