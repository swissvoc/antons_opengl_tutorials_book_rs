use std::fs::File;
use std::io::{BufRead, BufReader};
use std::mem;


pub fn load_obj_file(
    file_name: &str, 
    points: &mut [f32], tex_coords: &mut[f32],
    normals: &mut [f32], point_count: &mut usize) -> bool {

    let mut current_unsorted_vp = 0;
    let mut current_unsorted_vt = 0;
    let mut current_unsorted_vn = 0;

    let file = File::open(file_name);
    if file.is_err() {
        eprintln!("ERROR: could not find file {}", file_name);
        
        return false;
    }

    let file = file.unwrap();
    let reader = BufReader::new(file);

    // First count points in file so we know how much mem to allocate.
    *point_count = 0;
    let mut unsorted_vp_count = 0;
    let mut unsorted_vt_count = 0;
    let mut unsorted_vn_count = 0;
    let mut face_count = 0;

    for line in reader.lines().map(|st| st.unwrap()) {
        let bytes = line.as_bytes();
        if bytes[0] == b'v' {
            if bytes[1] == b' ' {
                unsorted_vp_count += 1;
            } else if bytes[1] == b't' {
                unsorted_vt_count += 1;
            } else if bytes[1] == b'n' {
                unsorted_vn_count += 1;
            }
        } else if bytes[0] == b'f' {
            face_count += 1;
        }
    }

    println!(
        "Found {} vp {} vt {} vn unique in obj. allocating memory...",
        unsorted_vp_count, unsorted_vt_count, unsorted_vn_count
    );

    let mut unsorted_vp_array = vec![0.0; 3 * unsorted_vp_count];
    let mut unsorted_vt_array = vec![0.0; 2 * unsorted_vt_count];
    let mut unsorted_vn_array = vec![0.0; 3 * unsorted_vn_count];

    println!("Allocated {} bytes for mesh", 3 * face_count * 8 * mem::size_of::<f32>());

    let file = File::open(file_name).unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines().map(|st| st.unwrap()) {
        // Vertex
        let bytes = line.as_bytes();
        if bytes[0] == b'v' {
            // Vertex point.
            if bytes[1] == b' ' {
                let (x, y, z) = scan_fmt!(&line, "v {} {} {}", f32, f32, f32);
                unsorted_vp_array[current_unsorted_vp * 3]     = x.unwrap();
                unsorted_vp_array[current_unsorted_vp * 3 + 1] = y.unwrap();
                unsorted_vp_array[current_unsorted_vp * 3 + 2] = z.unwrap();
                current_unsorted_vp += 1;

            // Vertex texture coordinate.
            } else if bytes[1] == b't' {
                let (s, t) = scan_fmt!(&line, "vt {} {}", f32, f32);
                unsorted_vt_array[current_unsorted_vt * 2]     = s.unwrap();
                unsorted_vt_array[current_unsorted_vt * 2 + 1] = t.unwrap();
                current_unsorted_vt += 1;

            // Vertex normal.
            } else if bytes[1] == b'n' {
                let (x, y, z) = scan_fmt!(&line, "vn {} {} {}", f32, f32, f32);
                unsorted_vn_array[current_unsorted_vn * 3]     = x.unwrap();
                unsorted_vn_array[current_unsorted_vn * 3 + 1] = y.unwrap();
                unsorted_vn_array[current_unsorted_vn * 3 + 2] = z.unwrap();
                current_unsorted_vn += 1;
            }

        // Faces
        } else if bytes[0] == b'f' {
            // work out if using quads instead of triangles and print a warning
            let mut slash_count = 0;
            for i in 0..bytes.len() {
                if bytes[i] == b'/' {
                    slash_count += 1;
                }
            }
            if slash_count != 6 {
                eprintln!(
                    "ERROR: file contains quads or does not match v vp/vt/vn layout - 
                     make sure exported mesh is triangulated and contains vertex points, 
                     texture coordinates, and normals"
                );
                
                return false;
            }

            let (vp0, vt0, vn0, vp1, vt1, vn1, vp2, vt2, vn2) = scan_fmt!(
                &line, "f {}/{}/{} {}/{}/{} {}/{}/{}", 
                usize, usize, usize, usize, usize, usize, usize, usize, usize
            );

            let vp = [vp0.unwrap(), vp1.unwrap(), vp2.unwrap()];
            let vt = [vt0.unwrap(), vt1.unwrap(), vt2.unwrap()];
            let vn = [vn0.unwrap(), vn1.unwrap(), vn2.unwrap()];

            // Start reading points into a buffer. order is -1 because 
            // obj starts from 1, not 0.
            // NB: assuming all indices are valid
            for i in 0..3 {
                if (vp[i] - 1 < 0 ) || (vp[i] - 1 >= unsorted_vp_count) {
                    eprintln!("ERROR: invalid vertex position index in face");
                    return false;
                }
                if (vt[i] - 1 < 0) || (vt[i] - 1 >= unsorted_vt_count) {
                    eprintln!("ERROR: invalid texture coord index {} in face.", vt[i]);
                    return false;
                }
                if (vn[i] - 1 < 0) || (vn[i] - 1 >= unsorted_vn_count) {
                    println!("ERROR: invalid vertex normal index in face");
                    return false;
                }

                points[*point_count * 3]     = unsorted_vp_array[(vp[i] - 1) * 3];
                points[*point_count * 3 + 1] = unsorted_vp_array[(vp[i] - 1) * 3 + 1];
                points[*point_count * 3 + 2] = unsorted_vp_array[(vp[i] - 1) * 3 + 2];
                
                tex_coords[*point_count * 2]     = unsorted_vt_array[(vt[i] - 1) * 2];
                tex_coords[*point_count * 2 + 1] = unsorted_vt_array[(vt[i] - 1) * 2 + 1];
                
                normals[*point_count * 3]     = unsorted_vn_array[(vn[i] - 1) * 3];
                normals[*point_count * 3 + 1] = unsorted_vn_array[(vn[i] - 1) * 3 + 1];
                normals[*point_count * 3 + 2] = unsorted_vn_array[(vn[i] - 1) * 3 + 2];
                
                *point_count += 1;
            }
        }
    }

    println!("Allocated {} points", point_count);
    
    return true;
}