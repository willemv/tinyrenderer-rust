use std::vec::Vec;
use std::path::Path;
use std::fs;

use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64
}

pub struct Face {
    pub vertex_indices: Vec<usize>,
    pub tex_indices: Vec<usize>
}
pub struct Model {
    pub vertices: Vec<Vec3>,
    pub tex_coords: Vec<Vec3>,
    pub faces: Vec<Face>
}

pub fn read_obj<P: AsRef<Path>>(path: P) -> Result<Model, ()> {
    let mut vertices = Vec::new();
    let mut tex_coords = Vec::new();
    let mut faces = Vec::new();

    let file = fs::File::open(path).unwrap();
    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        if line.starts_with("v ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let vec3 = Vec3 {
                x: parts[1].parse().unwrap(),
                y: parts[2].parse().unwrap(),
                z: parts[3].parse().unwrap()
            };
            vertices.push(vec3);
        } else if line.starts_with("vt ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let vec3 = Vec3 {
                x: parts[1].parse().unwrap(),
                y: parts[2].parse().unwrap(),
                z: parts[3].parse().unwrap()
            };
            tex_coords.push(vec3);
        } else if line.starts_with("f ") {
            let mut indices = Vec::new();
            let mut tex_indices = Vec::new();
            let parts: Vec<&str> = line.split_whitespace().collect();

            for part in parts {
                if part.starts_with("f") {continue}
                let subparts: Vec<&str> = part.split("/").collect();
                let index: usize = subparts[0].parse().unwrap();
                let tex_index: usize = subparts[1].parse().unwrap();
                indices.push(index - 1); //obj is 1-based, rust vecs 0-based
                tex_indices.push(tex_index -1);

            }
            faces.push(Face { vertex_indices: indices, tex_indices });
        }
    }

    Ok(Model{vertices, tex_coords, faces})
}