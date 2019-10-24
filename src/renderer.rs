use crate::tga::{TgaColor, TgaImage, rgb};
use crate::obj::Model;
use crate::obj::Vec3;

use std::cmp;

pub fn line(x0: u16, y0: u16, x1: u16, y1: u16, image: &mut TgaImage, color: &TgaColor) {
    let mut x0 = x0 as i32;
    let mut x1 = x1 as i32;
    let mut y0 = y0 as i32;
    let mut y1 = y1 as i32;

    let mut steep = false;
    if (x0-x1).abs() < (y0-y1).abs()  {
        std::mem::swap(&mut x0, &mut y0);
        std::mem::swap(&mut x1, &mut y1);
        steep = true;
    }
    if x0 > x1 {
        std::mem::swap(&mut x0, &mut x1);
        std::mem::swap(&mut y0, &mut y1);
    }

    let dx = x1-x0;
    let dy = y1-y0;
    let derror2 = dy.abs()  * 2;
    let mut error2 = 0;
    let mut y = y0 as u16;

    let x0 = x0 as u16;
    let x1 = x1 as u16;

    for x in x0 ..x1 {
        if steep {
            image.set(y, x, &color);
        } else {
            image.set(x, y, &color);
        }
        error2 += derror2;
        if error2 > dx {
            if y1 > y0 {y += 1} else {y -= 1}
            error2 -= dx * 2;
        }
    }
}

fn cross_product(x0: i32, y0: i32, z0: i32, x1: i32, y1: i32, z1: i32) -> (i64, i64, i64) {
    let x0 = x0 as i64;
    let y0 = y0 as i64;
    let z0 = z0 as i64;
    let x1 = x1 as i64;
    let y1 = y1 as i64;
    let z1 = z1 as i64;

    (
        y0 * z1 - z0 * y1,
        z0 * x1 - x0 * z1,
        x0 * y1 - y0 * x1
    )
}


fn cross_product_f(x0: f64, y0: f64, z0: f64, x1: f64, y1: f64, z1: f64) -> (f64, f64, f64) {
    (
        y0 * z1 - z0 * y1,
        z0 * x1 - x0 * z1,
        x0 * y1 - y0 * x1
    )
}

pub fn triangle(image: &mut TgaImage, zbuffer: &mut [f64],
                x0: u16, y0: u16,  z0: f64, t0: &Vec3,
                x1: u16, y1: u16,  z1: f64, t1: &Vec3,
                x2: u16, y2: u16,  z2: f64, t2: &Vec3,
                texture: &TgaImage,
                color: &TgaColor) {

    let x0 = x0 as i32;
    let y0 = y0 as i32;
    let x1 = x1 as i32;
    let y1 = y1 as i32;
    let x2 = x2 as i32;
    let y2 = y2 as i32;

    let xmin = cmp::min(x0, cmp::min(x1, x2));
    let xmax = cmp::max(x0, cmp::max(x1, x2));
    let ymin = cmp::min(y0, cmp::min(y1, y2));
    let ymax = cmp::max(y0, cmp::max(y1, y2));

    for xp in xmin..xmax {
        for yp in ymin..ymax {
            let cp = cross_product(x1 - x0, x2 - x0, x0 - xp,
                                   y1 - y0, y2 - y0, y0 - yp);
            let cpu = (cp.0 * cp.2, cp.1 * cp.2, cp.2 * cp.2);
            let inside = cpu.0 >= 0 && cpu.1 >= 0 && (cpu.0 + cpu.1 <= cpu.2);

            let u = cpu.0 as f64 / cpu.2 as f64;
            let v = cpu.1 as f64 / cpu.2 as f64;
            let index: usize = (xp + image.width as i32 * yp) as usize;
            let zp = bilerp(z0, z1, u, z2, v);
            let zb = zbuffer[index];

            if inside && zp > zb {
                zbuffer[index] = zp;

                let tpx = bilerp(t0.x, t1.x, u, t2.x, v);
                let tpy = bilerp(t0.y, t1.y, u, t2.y, v);


                let tpx = (tpx * texture.width as f64) as u16;
                let tpy = (tpy * texture.height as f64) as u16;

                let tpy = texture.height - tpy;
                let tex_color = texture.get(tpx, tpy);

                let multiplied_color = tex_color * color;

                image.set(xp as u16, yp as u16, &multiplied_color);
            }
        }
    }
}

fn dot_product(v0: (f64,f64,f64), v1: (f64,f64,f64)) -> f64 {
      v0.0 * v1.0
    + v0.1 * v1.1
    + v0.2 * v1.2
}

fn bilerp(v0: f64, v1: f64, fraction1: f64, v2: f64, fraction2: f64) -> f64 {
    return v0 + (v1 - v0) * fraction1 + (v2- v0) *fraction2;
}

fn normalize(v: (f64, f64, f64)) -> (f64, f64, f64) {
    let length = (v.0*v.0 + v.1*v.1 + v.2*v.2).sqrt();
    (v.0 / length, v.1 / length, v.2 / length)
}

pub fn render_model(mut image: &mut TgaImage,
                    texture: &TgaImage,
                    white: &TgaColor,
                    content_width: f64, content_height: f64,
                    center_x: f64, center_y: f64,
                    model: &Model) {
    let light_direction = (0.0, 0.0, -1.0);
    let data_size = image.width as usize * image.height as usize;
    let mut zbuffer = vec![std::f64::NEG_INFINITY; data_size].into_boxed_slice();


    for face in &model.faces {
        let vertex_count = face.vertex_indices.len();
        if vertex_count == 3 {
            let p0 = &model.vertices[face.vertex_indices[0]];
            let p1 = &model.vertices[face.vertex_indices[1]];
            let p2 = &model.vertices[face.vertex_indices[2]];

            let t0 = &model.tex_coords[face.tex_indices[0]];
            let t1 = &model.tex_coords[face.tex_indices[1]];
            let t2 = &model.tex_coords[face.tex_indices[2]];


            let normal = cross_product_f(p2.x -p0.x, p2.y- p0.y, p2.z - p0.z,
                                         p1.x -p0.x, p1.y- p0.y, p1.z - p0.z);
            let normal = normalize(normal);

            let intensity = dot_product(normal, light_direction);
            if intensity < 0.0 {
                continue;
            };

            let c = (intensity * 255.0) as u8;
            let color = &rgb(c, c, c);


            let x0 = center_x + (content_width / 2.0) * p0.x;
            let y0 = center_y + (content_height / 2.0) * p0.y;
            let z0: f64 = p0.z;

            let x1 = center_x + (content_width / 2.0) * p1.x;
            let y1 = center_y + (content_height / 2.0) * p1.y;
            let z1: f64 = p1.z;

            let x2 = center_x + (content_width / 2.0) * p2.x;
            let y2 = center_y + (content_height / 2.0) * p2.y;
            let z2: f64 = p2.z;

            triangle(image,
                     &mut zbuffer,
                     x0 as u16, y0 as u16, z0, t0,
                     x1 as u16, y1 as u16, z1, t1,
                     x2 as u16, y2 as u16, z2, t2,
                     texture,
                     color);
        }
        else if vertex_count > 2 {
            for i in 0..vertex_count {
                let p0 = &model.vertices[face.vertex_indices[i]];
                let p1 = &model.vertices[face.vertex_indices[(i + 1) % vertex_count]];

                let x0 = center_x + (content_width / 2.0) * p0.x;
                let y0 = center_y + (content_height / 2.0) * p0.y;
                let x1 = center_x + (content_width / 2.0) * p1.x;
                let y1 = center_y + (content_height / 2.0) * p1.y;

                line(x0 as u16, y0 as u16, x1 as u16, y1 as u16,
                               &mut image, &white);
            }
        }
    }
}
