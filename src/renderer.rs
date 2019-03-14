use crate::tga::{TgaColor, TgaImage, rgb};
use crate::obj::Model;

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

pub fn triangle(image: &mut TgaImage, x0: u16, y0: u16, x1: u16, y1: u16, x2: u16, y2: u16, color: &TgaColor) {
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

    let xp = (x0 + x1 + x2) / 3;
    let yp = (y0 + y1 + y2) / 3;

    for xp in xmin..xmax {
        for yp in ymin..ymax {
            let cp = cross_product((x1 - x0), (x2 - x0), (x0 - xp),
                                   (y1 - y0), (y2 - y0), (y0 - yp));
            let cpu = (cp.0 * cp.2, cp.1 * cp.2, cp.2 * cp.2);
            let inside = cpu.0 >= 0 && cpu.1 >= 0 && (cpu.0 + cpu.1 <= cpu.2);

            if inside {
                image.set(xp as u16, yp as u16, color);
            }
        }
    }
}

fn pseudo_random_colors() -> Vec<TgaColor> {
    let dr = 29;
    let dg = 13;
    let db = 31;

    let mut r = 201;
    let mut g = 23;
    let mut b = 31;

    let number_of_colors = 50;
    let mut colors = Vec::with_capacity(number_of_colors);
    for i in 0..number_of_colors {
        colors.push(rgb(r, g, b));
        r = r.wrapping_add(dr);
        g = g.wrapping_sub(dg);
        b = b.wrapping_add(db);
    }

    colors
}

pub fn render_model(mut image: &mut TgaImage, white: &TgaColor, content_width: f64, content_height: f64, center_x: f64, center_y: f64, model: &Model) {

    let colors = pseudo_random_colors();

    for (fi, face) in model.faces.iter().enumerate() {
        let vertex_count = face.vertex_indices.len();
        if vertex_count == 3 {
            let p0 = &model.vertices[face.vertex_indices[0]];
            let p1 = &model.vertices[face.vertex_indices[1]];
            let p2 = &model.vertices[face.vertex_indices[2]];

            let x0 = center_x + (content_width / 2.0) * p0.x;
            let y0 = center_y + (content_height / 2.0) * p0.y;
            let x1 = center_x + (content_width / 2.0) * p1.x;
            let y1 = center_y + (content_height / 2.0) * p1.y;
            let x2 = center_x + (content_width / 2.0) * p2.x;
            let y2 = center_y + (content_height / 2.0) * p2.y;

            let color_index = fi % colors.len();
            let color = colors.get(color_index).unwrap();

            triangle(&mut image,
                     x0 as u16, y0 as u16,
                     x1 as u16, y1 as u16,
                     x2 as u16, y2 as u16,
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
