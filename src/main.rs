use std::io::{Read, Write};

mod hilbert;

fn main() {

	// Read the location of the executable
	let current_exe = std::env::current_exe().unwrap();
	let exe_dir = current_exe.parent().unwrap();
	println!("exe dir: {:?}", exe_dir);

	// Check if the input and output directories exist.
	// If they don't, create new ones
	let bin2png_input = exe_dir.join("bin2png_input");
	let bin2png_output = exe_dir.join("bin2png_output");
	let png2bin_input = exe_dir.join("png2bin_input");
	let png2bin_output = exe_dir.join("png2bin_output");

	if bin2png_input.is_dir() == false {
		std::fs::create_dir_all(&bin2png_input).unwrap();
	}
	if bin2png_output.is_dir() == false {
		std::fs::create_dir_all(&bin2png_output).unwrap();
	}
	if png2bin_input.is_dir() == false {
		std::fs::create_dir_all(&png2bin_input).unwrap();
	}
	if png2bin_output.is_dir() == false {
		std::fs::create_dir_all(&png2bin_output).unwrap();
	}

	// Read input files
	let paths = std::fs::read_dir(bin2png_input.clone()).unwrap();
	for path in paths {
		let p = path.unwrap().path();
		// Ignore subdirectories
		if p.is_file() {
			// Read all bytes
			let file = match std::fs::File::open(&p) {
				Ok(f) => f,
				Err(_) => {
					println!("Failed to open a file ({:?}).", p);
					continue;
				}
			};
			let mut reader = std::io::BufReader::new(file);
			let mut bytes = Vec::new();
			for _ in 0..4 {
				bytes.push(0); // push 4 zeores, they will be filled with the length of the buffer later
			}
			let bytes_len = match reader.read_to_end(&mut bytes) {
				Ok(b) => b,
				Err(_) => {
					println!("Failed to read a file ({:?})", p);
					continue;
				}
			};
			let le_bytes = (bytes_len as u32).to_le_bytes();
			for elem in 0..4 {
				bytes[elem] = le_bytes[elem];
			}

			// Once we know the amount of bytes, we can start encoding them to an image!
			let mut hilbert_colors = Vec::<u8>::new();
			let pixel_count = 1 + (bytes_len + 4) / 3; // calculate the minimum amount of pixels needed for the read data (each pixel is 3 bytes);

			// Figure out dimensions of the image.
			// We want the image size to be a power of two (it creates prettier images!)
			// We also want it to be a square.
			// So we'll left-shift the value until it's big enough to contain all of the bytes.
			let mut edge = 1;
			while edge * edge < pixel_count {
				edge <<= 1; // this is equivalent to multiplying by two, becuase edge is usize
			}
			let edge = edge; // drop mutability

			// Loop over all bytes of the final image
			for elem in 0..edge*edge*4 {
				let color_coord = elem % 4;
				if color_coord == 3 {
					hilbert_colors.push(255);
					continue; // skip alpha coordinates
				}
				// figure out coordinates on the image
				let pixel_id = elem / 4;
				let x = pixel_id % edge;
				let y = pixel_id / edge;

				// use the hilbert-curve to figure out the corresponding byte
				let byte_index = hilbert::xy2d(edge, x, y) * 3;
				let byte_index = byte_index + color_coord;

				// write data, loop the buffer in case there aren't enough bytes -- it leads to fractal-like visuals
				let byte = bytes[byte_index % bytes.len()];
				hilbert_colors.push(byte);
			}

			// Get the output file.
			let mut output_path = bin2png_output.join(p.file_name().unwrap());
			// Add ".png" to the file path
			match p.extension() {
				Some(val) => {
					output_path.set_extension(format!("{}.png", val.to_str().unwrap()));
				},
				None => {
					output_path.set_extension("png");
				}
			};
			// Create the file if it doesn't exist yet
			match std::fs::File::create(&output_path) {
				Ok(_) => { },
				Err(_) => {
					println!("Failed to create file: ({:?})", output_path);
					continue;
				}
			};
			// Save the image
			match lodepng::encode32_file(&output_path, &hilbert_colors, edge, edge) {
				Ok(_) => {
					println!("Saved image: ({:?})", output_path)
				},
				Err(_) => {
					println!("Failed to save an image ({:?})", output_path);
				}
			};
		}
	}

	let paths = std::fs::read_dir(png2bin_input.clone()).unwrap();
	for path in paths {
		let p = path.unwrap().path();
		// Ignore subdirectories
		if p.is_file() {
			let image = match lodepng::decode32_file(&p) {
				Ok(im) => im,
				Err(_) => { 
					println!("Failed to read an image file ({:?}). Are you sure it's a png?", p);
					continue;
				}
			};
			if image.height != image.height {
				println!("Image ({:?}) is not a square. We can't decode it!", p);
				continue;
			}
			let image_bytes : Vec<lodepng::RGBA> = image.buffer;

			// First four bytes encode the size of the original data
			let mut b : [u8; 4] = [0, 0, 0, 0];
			for elem in 0..4 {
				let color_index = elem % 3;
				let pixel_index = elem / 3;
				let (x, y) = hilbert::d2xy(image.height, pixel_index);

				b[elem] = match color_index {
					0 => image_bytes[x + y * image.height].r,
					1 => image_bytes[x + y * image.height].g,
					2 => image_bytes[x + y * image.height].b,
					_ => image_bytes[x + y * image.height].r
				};
			}
			let size = u32::from_le_bytes(b) as usize; // size of the original binary



			let mut decoded_binary : Vec<u8> = vec![];

			for elem in 4..4+size {
				let color_index = elem % 3;
				let pixel_index = elem / 3;
				let (x, y) = hilbert::d2xy(image.height, pixel_index);

				decoded_binary.push(match color_index {
					0 => image_bytes[x + y * image.height].r,
					1 => image_bytes[x + y * image.height].g,
					2 => image_bytes[x + y * image.height].b,
					_ => image_bytes[x + y * image.height].r
				});
			}

			// Get the output file.
			let mut output_path = png2bin_output.join(p.file_name().unwrap());
			// Remove extension from the file
			output_path.set_extension("");
			// Open the file, truncating it in the process
			let mut file = match std::fs::File::create(&output_path) {
				Ok(f) => f,
				Err(_) => {
					println!("Failed to create file: ({:?})", output_path);
					continue;
				}
			};
			// Write decoded binary
			match file.write_all(&decoded_binary) {
				Ok(_) => {
					println!("Saved file {:?}", output_path);
				},
				Err(_) => {
					println!("Failed to save a file {:?}", output_path);
				}
			};
		}
	}

	println!("DONE!");
}
