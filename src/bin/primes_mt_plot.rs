use std::f64::consts::PI;
use std::time::Instant;
use std::{f64, thread};
use std::sync::{Arc, Mutex};
use image::{ImageBuffer, Rgb};
use clap::{Parser, ValueEnum};


#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum RingMode {
	/// Enable full ring (no angular sector skipping) when origin is in viewport or twin-coloring is used
	Auto,
	/// Always use sector mode, never fall back to full ring, even with twin-coloring
	Off,
	/// Always use full ring, regardless of viewport or coloring mode
	On,
}

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
	#[arg(short = 'x', long, default_value_t = 0.0, allow_hyphen_values=true)]
	center_bias_x: f64,

	/// Center bias Y coordinate
	#[arg(short = 'y', long, default_value_t = 0.0, allow_hyphen_values=true)]
	center_bias_y: f64,

	/// Fixed pixel size (overrides pixel_grow when != 1.0)
	#[arg(short = 'f', long, default_value_t = 1.0)]
	pixel_fixed_size: f64,

	/// Ring mode: auto=full ring when origin is visible or colored=1, off=always sector mode, on=always full ring
	#[arg(short = 'm', long, value_enum, default_value_t = RingMode::Auto)]
	ring_mode: RingMode,

	/// Output image path or filename (overrides the generated filename)
	#[arg(short = 'o', long)]
	file: Option<String>,
}


const TWO_PI: f64 = 2.0 * PI;

fn norm_angle(a: f64) -> f64 {
    a.rem_euclid(TWO_PI) // always in [0, 2π)
}

/// Next representable f64 toward +∞ (1 ULP up), without losing precision.
fn next_up(x: f64) -> f64 {
    if x.is_nan() || x == f64::INFINITY {
        return x;
    }
    // Handle both +0.0 and -0.0
    if x == 0.0 {
        return f64::from_bits(1); // smallest positive subnormal
    }
    let b = x.to_bits();
    // For positive numbers, increment bits; for negative numbers, decrement bits.
    if x > 0.0 {
        f64::from_bits(b + 1)
    } else {
        f64::from_bits(b - 1)
    }
}

/// Next representable f64 toward -∞ (1 ULP down), without losing precision.
fn next_down(x: f64) -> f64 {
    if x.is_nan() || x == f64::NEG_INFINITY {
        return x;
    }
    if x == 0.0 {
        // most negative subnormal
        return f64::from_bits(1u64 << 63 | 1);
    }
    let b = x.to_bits();
    // For positive numbers, decrement bits; for negative numbers, increment bits.
    if x > 0.0 {
        f64::from_bits(b - 1)
    } else {
        f64::from_bits(b + 1)
    }
}

fn pretty_print_int(i: usize) -> String {
	let mut s = String::new();
	let i_str = i.to_string();
	let a = i_str.chars().rev().enumerate();
	for (idx, val) in a {
		if idx != 0 && idx % 3 == 0 {
			s.insert(0, ' ');
		}
		s.insert(0, val);
	}
	s
}


fn is_prime(n:&usize)->bool{
	let limit = (*n as f64).sqrt() as usize +1;
	let mut divisor = 3;

	while divisor <= limit {
		if n % divisor == 0 {
			return false;
		}
		divisor += 2;
	}
	true
}


// struct Coord {
// 	x: f64,
// 	y: f64
// }


/// Returns coordinates for all 
// fn get_square_vertex(center: &Coord, half_size: &f64) -> [Coord; 4] {
// 	[
// 		Coord { x: center.x - half_size, y: center.y - half_size },
// 		Coord { x: center.x + half_size, y: center.y - half_size },
// 		Coord { x: center.x + half_size, y: center.y + half_size },
// 		Coord { x: center.x - half_size, y: center.y + half_size },
// 	]
// }


/// Evaluates if a given coordinate is inside of a given square
// fn is_inside_square(point: &Coord, square: &[Coord; 4]) -> bool {
// 	let mut min_x = square[0].x;
// 	let mut max_x = square[0].x;
// 	let mut min_y = square[0].y;
// 	let mut max_y = square[0].y;

// 	for v in square.iter().skip(1) {
// 		if v.x < min_x { min_x = v.x; }
// 		if v.x > max_x { max_x = v.x; }
// 		if v.y < min_y { min_y = v.y; }
// 		if v.y > max_y { max_y = v.y; }
// 	}

// 	point.x >= min_x && point.x <= max_x && point.y >= min_y && point.y <= max_y
// }


/// Calculates the length of a vector using x,y coordinates
// fn get_vector_length(point: &Coord) -> f64 {
// 	(point.x.powi(2) + point.y.powi(2)).sqrt()
// }


/// Calculates the start and end number (prime/world units) needed to cover the viewport.
/// Inputs:
/// - `scale`: pixels per 1.0 world unit (prime radius unit)
/// - `offset_x_px`, `offset_y_px`: viewport center bias in pixels
/// - `half_size_px`: half of viewport width/height in pixels (i.e., image_size/2)
fn get_calculation_ring(
	scale: &f64,
	offset_x_px: &f64,
	offset_y_px: &f64,
	half_size_px: &f64,
) -> [usize; 2] {
	// Convert pixel-space viewport definition into world-space (prime radius units)
	let cx = offset_x_px / scale;
	let cy = offset_y_px / scale;
	let h = half_size_px / scale;

	// Exact min distance from origin to an axis-aligned square (AABB) centered at (cx, cy) with half-size h
	let ax = cx.abs();
	let ay = cy.abs();
	let dx = (ax - h).max(0.0);
	let dy = (ay - h).max(0.0);
	let min_dist = (dx * dx + dy * dy).sqrt();

	// Safe max distance: farthest corner distance
	let max_dist = ((ax + h).powi(2) + (ay + h).powi(2)).sqrt();

	// Convert to integer boundaries for prime scanning
	let mut start = min_dist.floor() as usize;
	if start < 3 { start = 3; }
	// keep start odd so threaded stepping (+= 2*num_threads) stays on odds
	if start % 2 == 0 {
		start = start.saturating_sub(1);
		if start < 3 { start = 3; }
	}

	let end = (max_dist.ceil() as usize).max(start);

	println!(
		"Ring boundaries: start={}, end={} (cx={}, cy={}, h={}, scale={})",
		pretty_print_int(start),
		pretty_print_int(end),
		pretty_print_int(cx as usize),
		pretty_print_int(cy as usize),
		pretty_print_int(h as usize),
		scale
	);

	[start, end]
}


fn get_radian(x: &f64, y: &f64) -> f64 {
    norm_angle(y.atan2(*x))
}

fn angle_in_arc(angle: f64, arc_min: f64, arc_max: f64) -> bool {
    // Supports wrap-around when arc_min > arc_max
    if arc_min <= arc_max {
        angle >= arc_min && angle <= arc_max
    } else {
        angle >= arc_min || angle <= arc_max
    }
}

/// returns min and max arc in radians and if square (viewport) contains origin (0,0)
/// `force_sector` skips the full-circle shortcut below, forcing a (possibly incomplete)
/// corner-based sector even when the origin is inside the viewport.
fn get_calculation_arc(
    scale: &f64,
    offset_x_px: &f64,
    offset_y_px: &f64,
    half_size_px: &f64,
    force_sector: bool,
) -> (f64, f64, bool) {
    // Keep sign consistent with get_calculation_ring() and your drawing center_bias usage.
    let cx = offset_x_px / scale;
    let cy = offset_y_px / scale;
    let h = half_size_px / scale;

    // True "origin is displayed" test: origin inside the viewport AABB in world coords
    let origin_is_displayed = cx.abs() <= h && cy.abs() <= h;
    if origin_is_displayed && !force_sector {
        println!("Arc boundaries: None (origin inside viewport)");
        return (0.0, TWO_PI, true);
    }

    let square = [
        [cx - h, cy - h],
        [cx - h, cy + h],
        [cx + h, cy + h],
        [cx + h, cy - h],
    ];

    for (i, v) in square.iter().enumerate() {
        println!("Viewport corner {}: {};{}", i + 1, v[0], v[1]);
    }

    let mut angles: Vec<f64> = square.iter().map(|v| get_radian(&v[0], &v[1])).collect();
    angles.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Largest-gap method (robust wrap handling via rem_euclid)
    let mut max_gap = -1.0_f64;
    let mut max_gap_idx = 0usize;

    for i in 0..angles.len() {
        let a = angles[i];
        let b = angles[(i + 1) % angles.len()];
        let gap = (b - a).rem_euclid(TWO_PI); // always >= 0, wrap-safe
        if gap > max_gap {
            max_gap = gap;
            max_gap_idx = i;
        }
    }

    // Minimal covering arc is the complement of the largest gap:
    // start right after the largest gap, end at the start of the gap.
    let mut arc_min = angles[(max_gap_idx + 1) % angles.len()];
    let mut arc_max = angles[max_gap_idx];

    // Expand by 1 ULP so comparisons don't miss points exactly on the boundary.
    // This keeps full f64 resolution (no coarse epsilon).
    arc_min = norm_angle(next_down(arc_min));
    arc_max = norm_angle(next_up(arc_max));

    if arc_min <= arc_max {
        println!("Arc boundaries: {}-{}", arc_min, arc_max);
    } else {
        println!("Arc boundaries (wrap): {}-{} (wraps over 0)", arc_min, arc_max);
    }

    (arc_min, arc_max, false)
}

fn main() {
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

	// Compute needed prime range for the current viewport (offsets are pixels; half-size is image_size/2 px)
	let half_size_px = image_size as f64 / 2.0;
	let boundaries = get_calculation_ring(&scale, &center_bias_x, &center_bias_y, &half_size_px);
	let calc_start = boundaries[0];
	let draw_radius = boundaries[1];

	let force_sector = args.ring_mode == RingMode::Off;
	let arc_boundaries = get_calculation_arc(&scale, &center_bias_x, &center_bias_y, &half_size_px, force_sector);
	let calc_arc_min = arc_boundaries.0;
	let calc_arc_max = arc_boundaries.1;
	let origin_is_displayed = arc_boundaries.2;

	// Ring mode calculates the full radial ring (no angular sector skipping), which is required
	// when the origin is in view (sector method would leave a big unscanned gap) and when
	// twin-prime coloring is used (a prime's pair can lie far outside the sector).
	// `off` and `on` are explicit overrides that always win over the origin/coloring auto-detection.
	let ring_mode_active = match args.ring_mode {
		RingMode::Off => false,
		RingMode::On => true,
		RingMode::Auto => origin_is_displayed || colored == 1,
	};

	if ring_mode_active {
		let reason = match args.ring_mode {
			RingMode::On => "forced by --ring-mode on",
			_ if origin_is_displayed => "origin is in viewport",
			_ => "colored=1 (twin primes)",
		};
		println!("Ring mode: ON ({})", reason);
	} else {
		println!("Ring mode: OFF (sector mode)");
	}


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
		let step = 2 * num_threads;
		
		let handle = thread::spawn(move || {
			let mut primes = Vec::new();
			let mut n = calc_start + 2 * i;
			
			let start_time_clone = Instant::now();

			if ring_mode_active {
				while start_time_clone.elapsed().as_secs_f64() < time_limit {
					if is_prime(&n) {
						primes.push(n);
					}
	
					n += step;
	
					if n > draw_radius{ break; }
				}
			}else{
				while start_time_clone.elapsed().as_secs_f64() < time_limit {
					let n_rad: f64 = (n as f64).rem_euclid(TWO_PI);

					if angle_in_arc(n_rad, calc_arc_min, calc_arc_max) {
						if is_prime(&n) {
							primes.push(n);
						}
					}

					n += step;
	
					if n > draw_radius{ break; }
				}
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
	let prime_counter = results.len();
	let first_prime = *results.iter().min().unwrap_or(&0);
	let last_prime = *results.iter().max().unwrap_or(&0);
	
	println!(
		"Found {} primes in {}s: {}..{}",
		pretty_print_int(prime_counter),
		start_time.elapsed().as_secs_f64(),
		pretty_print_int(first_prime),
		pretty_print_int(last_prime),
	);


	// if to display colored pairs - presort results vector
	if colored == 1 {
		println!("Sorting...");
		results.sort();
	}
	let results_length = results.len();


	// Generate polar plot
	println!("Generating polar plot image {}px", image_size);
	
	let mut img = ImageBuffer::from_pixel(image_size, image_size, Rgb([0u8, 0u8, 0u8]));

	// center_bias is subtracted (not added) here to match the world<->pixel convention used by
	// get_calculation_ring()/get_calculation_arc(), where offset_x_px/offset_y_px map directly
	// (unnegated) to the viewport's world-space center. x/y below then land directly in [0, image_size).
	let center_x = image_size as f64 / 2.0 - center_bias_x;
	let center_y = image_size as f64 / 2.0 - center_bias_y;
	let mut pixel = (255u8, 255u8, 255u8);
	let mut drawn = 0;
	let start_draw_time = Instant::now();

	for &prime in results.iter() {
		let angle = prime as f64;
		let radius = angle * scale;
		
		let x = center_x + radius * angle.cos();
		let y = center_y + radius * angle.sin();
		
		if 
		x >= 0.0
		&& x < image_size as f64
		&& y >= 0.0
		&& y < image_size as f64
		{
			drawn += 1;

			let px = x as i32;
			let py = y as i32;

			if colored > 1 {
				pixel = match (prime % 10) as u8 {
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

				pixel = match (trailing, leading) {
					(true, false) => (0u8, 255u8, 0u8),  // trailing only - green
					(false, true) => (255u8, 0u8, 0u8),  // leading only - red
					_ => (50u8, 50u8, 50u8)           // no neighbors, or both neighbours - grey
				}

			}

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
	

	let filename = args.file.unwrap_or_else(|| format!(
		"{}K_primes_{}_rad_{}_grow_{}_color_{}_x_{}_y_{}.png",
		image_size / 1000, drawn, max_radius, pixel_grow, colored, center_bias_x, center_bias_y,
	));
	img.save(&filename).expect("Failed to save image");
	println!("Drawn {} points in {}s", drawn, start_draw_time.elapsed().as_secs_f64());
	println!("Saved as {}", filename);
}