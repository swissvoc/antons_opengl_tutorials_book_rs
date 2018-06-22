use png;
use png::HasParameters;

use chrono::prelude::Utc;

use std::path::Path;
use std::fs::File;
use std::io::BufWriter;


pub fn capture<F>(height: usize, width: usize, depth: usize, capture_func: &F) -> bool 
    where F: Fn(&mut [u8]) -> bool
{
    let mut image_buffer: Vec<u8> = vec![0; (height * width * depth) as usize];
    
    // Capture the buffer data from the source and write it into the 
    // image buffer.
    let result = capture_func(&mut image_buffer);
    if !result {
        return false;
    }

    let width_in_bytes = depth * width;
    let half_height = height / 2;
    for row in 0..half_height {
        for col in 0..width_in_bytes {
            let temp = image_buffer[row * width_in_bytes + col];
            image_buffer[row * width_in_bytes + col] = image_buffer[((height - row - 1) * width_in_bytes) + col];
            image_buffer[((height - row - 1) * width_in_bytes) + col] = temp;
        }
    }

    let date = Utc::now();
    let name = format!("screenshot_{}.png", date);
    
    let path = Path::new(&name);
    let file = File::create(path).unwrap();
    let buf_writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(buf_writer, width as u32, height as u32);
    encoder.set(png::ColorType::RGB).set(png::BitDepth::Eight);
    let mut png_writer = encoder.write_header().unwrap();
    
    let result =  png_writer.write_image_data(&image_buffer);
    if result.is_err() {
        return false;
    }

    true
}
