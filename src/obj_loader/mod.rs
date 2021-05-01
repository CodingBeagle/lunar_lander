// Modules are ways of organizing code within a crate for easy reuse and readability.
// Modules also control the privacy of items. They will be private by default.
// Modules are defined using the "mod" keyword.
use std::{collections::HashMap, path::PathBuf};
use std::io::*;
use std::fs::File;

// io::prelude is a module that alleviates imports of many common I/O traits
use std::io::prelude::*;

use ultraviolet::vec::*;

#[repr(C)]
pub struct Vertex {
    pub position: Vec3,
    pub uv: Vec2,
    pub color: Vec4
}

#[derive(Default)]
pub struct ObjLoaderResult {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<i32>
}

pub fn load_obj(file_path: PathBuf) -> ObjLoaderResult {
    // .expect is a method on the Result enum which will return the OK value in case of success
    // And otherwise automaticlaly panic! in case of failure.a
    // First I get a handle to a file on the filesystem
    let mut obj_file = File::open(file_path).expect("failed to read obj file");

    // Then I create a buffered reader to the file using the file handle.
    let buff_read = BufReader::new(obj_file);

    // Lastly, I get an iterator to all lines in the file.
    let lines = buff_read.lines();

    let mut vertices : Vec<Vec3> = Vec::new();
    let mut uv : Vec<Vec2> = Vec::new();
    let mut indices : Vec<i32> = Vec::new();

    let mut vertex_objs : Vec<Vertex> = Vec::new();

    let mut rofl : HashMap<String, usize> = HashMap::new();

    for current_line_result in lines {
        let current_line = current_line_result.expect("Failed to fetch line from obj file.");

        // TODO: Gotta read up on that difference between String and &str...
        let parts : Vec<&str> = current_line.split_whitespace().collect();

        // Starting with "v", we got a vertex line
        if parts[0] == "v" {
            vertices.push( Vec3::new(
                    parts[1].parse::<f32>().expect("Failed to convert vertice string to number"),
                    parts[2].parse::<f32>().expect("Failed to convert vertice string to number"),
                    parts[3].parse::<f32>().expect("Failed to convert vertice string to number")));
         };
        

        // Starting with "vt" we got a texture coordinate
        if parts[0] == "vt" {
            uv.push(Vec2::new(
                    parts[1].parse::<f32>().expect("Failed to convert UV u coordinate."),
                    parts[2].parse::<f32>().expect("Failed to convert UV v coordinate.")));
        }

        // Starting with "f", we get a face element
        // A face can contain three OR MORE vertices
        if parts[0] == "f" {
            for n in 1..parts.len() {
                let the_part = parse_face_element(parts[n]);

                // Does vertex for this combination already exist
                let haha = format!("{},{},{}", the_part[0], the_part[1], the_part[2]);
                // println!("{}", haha);
                match rofl.get(&haha) {
                    Some(val) => {
                        let the_value = *val;
                        indices.push(the_value as i32)
                    },
                    None => {
                        vertex_objs.push( Vertex {
                            position: vertices[(the_part[0] - 1) as usize],
                            uv: uv[(the_part[1] - 1) as usize],
                            color: Vec4::default()
                         });

                        let new_index = vertex_objs.len() - 1;

                        rofl.entry(haha).or_insert(new_index);

                        indices.push(new_index as i32);
                    }
                }
            }
        }
    }

    println!("THE COUNT {}", vertex_objs.len());

    ObjLoaderResult {
        vertices: vertex_objs,
        indices
    }
}

fn parse_face_element(face_element: &str) -> Vec<i32> {
    face_element
        .split('/')
        .map(|element| element.parse::<i32>().expect("Failed to parse element of face from obj line."))
        .collect()
}
