use std::fs::File;
use std::io;
use std::io::{Seek, SeekFrom, BufRead, BufReader};
use std::mem;


#[derive(Clone, Debug, PartialEq)]
pub struct ObjMesh {
    pub point_count: usize,
    pub points: Vec<f32>,
    pub tex_coords: Vec<f32>,
    pub normals: Vec<f32>,
}


fn skip_spaces(bytes: &[u8]) -> usize {
    let mut index = 0;
    while index < bytes.len() - 1 { 
        if bytes[index] == b' ' || bytes[index] == b'\\' {
            index += 1;
        } else {
            break;
        }
    }

    index
}

fn count_vertices<T: BufRead + Seek>(reader: &mut T) -> (usize, usize, usize, usize) {
    let mut unsorted_vp_count = 0;
    let mut unsorted_vt_count = 0;
    let mut unsorted_vn_count = 0;
    let mut face_count = 0;

    for line in reader.lines().map(|st| st.unwrap()) {
        let bytes = line.as_bytes();
        let i = skip_spaces(bytes);
        if bytes[i] == b'v' {
            if bytes[i+1] == b' ' {
                unsorted_vp_count += 1;
            } else if bytes[i+1] == b't' {
                unsorted_vt_count += 1;
            } else if bytes[i+1] == b'n' {
                unsorted_vn_count += 1;
            }
        } else if bytes[i] == b'f' {
            face_count += 1;
        }
    }

    (unsorted_vp_count, unsorted_vt_count, unsorted_vn_count, face_count)
}

pub fn load_obj_mesh<T: BufRead + Seek>(reader: &mut T) -> io::Result<ObjMesh> {
    let mut current_unsorted_vp = 0;
    let mut current_unsorted_vt = 0;
    let mut current_unsorted_vn = 0;

    // First count points in file so we know how much mem to allocate.
    let (unsorted_vp_count, unsorted_vt_count, unsorted_vn_count, face_count) = count_vertices(reader);

    println!(
        "Found {} vp {} vt {} vn unique in obj. allocating memory...",
        unsorted_vp_count, unsorted_vt_count, unsorted_vn_count
    );

    println!("Found {} Faces.", face_count);

    let mut unsorted_vp_array = vec![0.0; 3 * unsorted_vp_count];
    let mut unsorted_vt_array = vec![0.0; 2 * unsorted_vt_count];
    let mut unsorted_vn_array = vec![0.0; 3 * unsorted_vn_count];

    println!("Allocated {} bytes for mesh", 3 * face_count * 8 * mem::size_of::<f32>());

    let mut points     = vec![];
    let mut tex_coords = vec![];
    let mut normals    = vec![];
    let mut point_count = 0;

    reader.seek(SeekFrom::Start(0)).unwrap();
    for line in reader.lines().map(|st| st.unwrap()) {
        // Vertex
        println!("line = \"{}\"", line);
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
                
                panic!()
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
                    panic!();
                }
                if (vt[i] - 1 < 0) || (vt[i] - 1 >= unsorted_vt_count) {
                    eprintln!("ERROR: invalid texture coord index {} in face.", vt[i]);
                    panic!();
                }
                if (vn[i] - 1 < 0) || (vn[i] - 1 >= unsorted_vn_count) {
                    println!("ERROR: invalid vertex normal index in face");
                    panic!();
                }

                points.push(unsorted_vp_array[(vp[i] - 1) * 3]);
                points.push(unsorted_vp_array[(vp[i] - 1) * 3 + 1]);
                points.push(unsorted_vp_array[(vp[i] - 1) * 3 + 2]);
                
                tex_coords.push(unsorted_vt_array[(vt[i] - 1) * 2]);
                tex_coords.push(unsorted_vt_array[(vt[i] - 1) * 2 + 1]);
                
                normals.push(unsorted_vn_array[(vn[i] - 1) * 3]);
                normals.push(unsorted_vn_array[(vn[i] - 1) * 3 + 1]);
                normals.push(unsorted_vn_array[(vn[i] - 1) * 3 + 2]);
                
                point_count += 1;
            }
        }
    }

    println!("Allocated {} points", point_count);
    
    Ok(ObjMesh {
        point_count: point_count,
        points: points,
        tex_coords: tex_coords,
        normals: normals,
    })
}

pub fn load_obj_file(file_name: &str) -> io::Result<ObjMesh> {
    let file = match File::open(file_name) {
        Ok(handle) => handle,
        Err(e) => {
            eprintln!("ERROR: could not find file {}", file_name);
            return Err(e);
        }
    };

    let mut reader = BufReader::new(file);
    load_obj_mesh(&mut reader)
}

mod parser_tests {
    use super::ObjMesh;
    use std::io::{BufReader, Cursor};

    struct Test {
        obj_file: String,
        obj_mesh: ObjMesh,   
    }

    fn test() -> Test {
        let obj_file = String::from(r"        \
            o object1                         \
            g cube                            \
            v  0.0  0.0  0.0                  \
            v  0.0  0.0  1.0                  \
            v  0.0  1.0  0.0                  \
            v  0.0  1.0  1.0                  \
            v  1.0  0.0  0.0                  \
            v  1.0  0.0  1.0                  \
            v  1.0  1.0  0.0                  \
            v  1.0  1.0  1.0                  \
                                              \
            vn  0.0  0.0  1.0                 \
            vn  0.0  0.0 -1.0                 \
            vn  0.0  1.0  0.0                 \
            vn  0.0 -1.0  0.0                 \
            vn  1.0  0.0  0.0                 \
            vn -1.0  0.0  0.0                 \
                                              \
            f  1//2  7//2  5//2               \
            f  1//2  3//2  7//2               \
            f  1//6  4//6  3//6               \
            f  1//6  2//6  4//6               \
            f  3//3  8//3  7//3               \
            f  3//3  4//3  8//3               \
            f  5//5  7//5  8//5               \
            f  5//5  8//5  6//5               \
            f  1//4  5//4  6//4               \
            f  1//4  6//4  2//4               \
            f  2//1  6//1  8//1               \
            f  2//1  8//1  4//1               \
        ");
        let point_count = 8;
        let points = vec![
            0.0, 0.0, 0.0,
            0.0, 0.0, 1.0,
            0.0, 1.0, 0.0,
            0.0, 1.0, 1.0,
            1.0, 0.0, 0.0,
            1.0, 0.0, 1.0,
            1.0, 1.0, 0.0,
            1.0, 1.0, 1.0,
        ];
        let tex_coords = vec![];
        let normals = vec![
             0.0,  0.0,  1.0,
             0.0,  0.0, -1.0,
             0.0,  1.0,  0.0,
             0.0, -1.0,  0.0,
             1.0,  0.0,  0.0,
            -1.0,  0.0,  0.0,
        ];

        let obj_mesh = ObjMesh {
            point_count: point_count,
            points: points,
            tex_coords: tex_coords,
            normals: normals,
        };

        Test {
            obj_file: obj_file,
            obj_mesh: obj_mesh,
        }
    }

    #[test]
    fn test_parse_obj_mesh() {
        let test = test();
        let result = super::load_obj_mesh(
            &mut BufReader::new(Cursor::new(test.obj_file.as_bytes()))
        ).unwrap();
        let expected = test.obj_mesh;

        assert_eq!(result, expected);
    }
}

