# Prime numbers polar plotter (tests)

A multi-threaded Rust program that finds prime numbers and generates a polar plot visualization.

![showcase](./example.jpg)

## Description

This program uses multiple threads to efficiently find prime numbers up to a specified limit, then creates a beautiful polar coordinate visualization where each prime number is plotted as a point. The angle represents the prime's value (in radians) and the radius also represents the prime's magnitude, creating distinctive spiral patterns.

## Features

- **Multi-threaded prime finding**: Automatically uses all available CPU cores (or specify a custom thread count)
- **Time-limited execution**: Run for a specific duration to find as many primes as possible
- **Polar plot visualization**: Creates a PNG image showing primes in polar coordinates
- **Configurable rendering**: Adjust image size, display radius, and point size for different visual effects
- **Gradient drawing**: Points grow larger toward the edge and use gradient rendering for smooth appearance
- **Color version**: Points might alter the color based on their last digit

![showcase in motion](./example.gif)

## Usage

For Windows:  
```bash
.\target\release\primes_mt_plot.exe [time_limit] [image_size] [max_radius] [pixel_grow] [threads] [colored]
Compile for other systems (optional):  
The compiled binary will be in `target/release/primes_mt_plot`.  

```bash
cargo build --release --bin primes_mt_plot
```

### Command-Line Arguments

All arguments are optional and positional:

1. **time_limit** (default: `10.0`)
   - Limits the time co collect prime nubmers. Search will stop earlyer, if all visualisation nessesary numbers are found. Or if hit the time limit before it, will plot only found numbers.
   - Type: float
   - Example: `30.0` for 30 seconds

2. **image_size** (default: `1000`)
   - Width and height of output image in pixels
   - Type: integer
   - Example: `2000` for a 2000×2000 image

3. **max_radius** (default: `100000.0`)
   - Maximum radius for visualization (determines which primes to display)
   - Type: float
   - Example: `50000.0` means at centers of image edges, will be numbers just below 50K, and up to 50K*sqrt(2) in corners

4. **pixel_grow** (default: `5.0`)
   - Point size growth factor (larger = bigger dots toward edge)
   - Type: float
   - Example: `10.0` for larger points, `1.0` for fixed size single-pixel points

5. **threads** (default: `0`)
   - Number of threads to use (0 = auto-detect)
   - Type: integer
   - Example: `8` to use exactly 8 threads

6. **colored** (default: `0`)
   - Enable multicolor visualization (0 = white/monochrome, non-zero = colored)
   - Type: integer
   - Colors primes based on their last digit:
     - Ending in 1: Cyan
     - Ending in 3: Magenta
     - Ending in 5: Yellow
     - Ending in 7: Green
     - Ending in 9: Blue
   - Example: `1` to enable colored visualization

## Examples

### Basic usage with defaults
```bash
.\target\release\primes_mt_plot.exe
```
Runs with 10 seconds limit, creates 1000×1000 image, shows primes up to 100,000

### High-quality large visualization
```bash
.\target\release\primes_mt_plot.exe 600 32000 2000000000 1 0 0
```
Runs with 600 seconds limit, creates 32000×32000 image, shows primes from 1 to 2_000_000_000

### Colored visualization
```bash
.\target\release\primes_mt_plot.exe 10 1000 100000 5 0 1
```
Creates a colorful visualization where primes are colored by their last digit

### Fast single-pixel rendering
```bash
.\target\release\primes_mt_plot.exe 10 1000 500000 1 0 0
```
Uses 1-pixel points (no gradient) for faster rendering

## Output

The program produces two types of output:

### Console Output
```
Starting max 10s run with 16 threads...
Found 78 498 primes in 10.000123s. Biggest is 999 983.
Generating polar plot image 1000px with max radius 100 000...
Saved polar plot to 1K_primes_78498_rad_100000_grow_5.png
```

### Image File
A PNG file named with the format:
```
{image_size}K_primes_{count}_rad_{max_radius}_grow_{pixel_grow}_color_{colored}.png
```
where count - how much numbers are actually rendered

Example: `1K_primes_78498_rad_100000_grow_5_color_0.png`

## How It Works

1. **Prime Finding**: The program distributes odd numbers across multiple threads, each checking for primality using trial division up to the square root
2. **Time Management**: Each thread monitors elapsed time and stops when the time limit is reached
3. **Result Aggregation**: All threads contribute their found primes to a shared, mutex-protected vector
4. **Visualization**: Each prime `p` is plotted at polar coordinates (angle=p, radius=p×scale)
5. **Gradient Rendering**: Points are drawn with a gradient that fades from center to edge, with size increasing based on distance from origin

## Performance Notes

- Always use `--release` flag for optimal performance
- More threads generally improve performance up to the number of physical CPU cores
- Larger images take longer to render, especially with high `pixel_grow` values
- Setting `pixel_grow=1.0` speeds up image generation scipping pixel size calculation
- Generated png images generally can be opened up to 32K-40K resolution. Larger images need specialized software to open like "vipsdisp" or others.
- Bigger max_radius -> bigger image size -> smaller pixel_grow (down to 1). To produce visually good, not to bright and not to dark images.

## Mathematical Background

The visualization uses polar coordinates where:
- **Angle (θ)**: The prime number itself (in radians)
- **Radius (r)**: The prime number scaled to fit the image

This creates the characteristic spiral patterns known as "Ulam spirals" or "prime spirals," revealing interesting structures in the distribution of prime numbers.

## Dependencies

- `image` crate: For PNG generation
- Standard library: Threading and synchronization

## Some examples
Colored with high pixel grow value:
![showcase](./example2.jpg)

Colored on higher scale
![showcase](./example3.png)

110M prime numbers plotted white-only. Around 4 minute build time
![showcase](./example4.png)