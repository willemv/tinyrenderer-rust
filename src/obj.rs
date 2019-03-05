use std::vec::Vec;
use std::path::Path;
use std::error::Error;
use std::fs;

use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64
}

pub struct Face {
    pub vertex_indices: Vec<usize>
}
pub struct Model {
    pub vertices: Vec<Vec3>,
    pub faces: Vec<Face>
}

pub fn read_obj<P: AsRef<Path>>(path: P) -> Result<Model, ()> {
    let mut vertices = Vec::new();
    let mut faces = Vec::new();

    let file = fs::File::open(path).unwrap();
    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        if line.starts_with("v ") {
            let parts: Vec<&str> = line.split(" ").collect();
            let vec3 = Vec3 { x: parts[1].parse().unwrap(), y: parts[2].parse().unwrap(), z: parts[3].parse().unwrap() };
            vertices.push(vec3);
        } else if line.starts_with("f ") {
            let mut indices = Vec::new();
            let parts: Vec<&str> = line.split(" ").collect();

            for part in parts {
                if part.starts_with("f") {continue}
                let subparts: Vec<&str> = part.split("/").collect();
                let index: usize = subparts[0].parse().unwrap();
                indices.push(index-1);
            }
            faces.push(Face { vertex_indices: indices });
        }
    }

    Ok(Model{vertices, faces})
}