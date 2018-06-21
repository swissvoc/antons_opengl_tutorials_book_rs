pub fn capture() -> bool {
    let height = unsafe { G_GL_HEIGHT as usize };
    let width = unsafe { G_GL_WIDTH as usize };
    let mut frame_buffer: Vec<u8> = vec![0; 3 * (width * height) as usize];
    unsafe {
        gl::ReadPixels(
            0, 0, G_GL_WIDTH as i32, G_GL_HEIGHT as i32, 
            gl::RGB, gl::UNSIGNED_BYTE, 
            frame_buffer.as_mut_ptr() as *mut GLvoid
        );
    }
    
    let width_in_bytes = 3 * width;
    let half_height = height / 2;
    for row in 0..half_height {
        for col in 0..width_in_bytes {
            let temp = frame_buffer[row * width_in_bytes + col];
            frame_buffer[row * width_in_bytes + col] = frame_buffer[((height - row - 1) * width_in_bytes) + col];
            frame_buffer[((height - row - 1) * width_in_bytes) + col] = temp;
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
    
    println!("Writing {}", name);
    
    let result =  png_writer.write_image_data(&frame_buffer);
    if result.is_err() {
        eprintln!("ERROR: could not write screenshot file {}", name);
    }

    return true;
}
