use crate::tga::{TgaColor, TgaImage};
use crate::obj::Model;

pub fn line(mut x0: u16, mut y0: u16, mut x1: u16,  mut y1: u16, image: &mut TgaImage, color: &TgaColor) {
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

pub fn render_model(mut generated: &mut TgaImage, white: &TgaColor, content_width: f64, content_height: f64, center_x: f64, center_y: f64, model: &Model) {
    for face in &model.faces {
        let vertex_count = face.vertex_indices.len();
        if vertex_count > 2 {
            for i in 0..vertex_count {
                let p0 = &model.vertices[face.vertex_indices[i]];
                let p1 = &model.vertices[face.vertex_indices[(i + 1) % vertex_count]];

                let x0 = center_x + (content_width / 2.0) * p0.x;
                let y0 = center_y + (content_height / 2.0) * p0.y;
                let x1 = center_x + (content_width / 2.0) * p1.x;
                let y1 = center_y + (content_height / 2.0) * p1.y;

                line(x0 as u16, y0 as u16, x1 as u16, y1 as u16,
                               &mut generated, &white);
            }
        }
    }
}
