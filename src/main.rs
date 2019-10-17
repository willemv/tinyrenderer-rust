mod tga;
mod obj;
mod renderer;

fn main() {
    let image_width = 800;
    let image_height = 800;
    let padding = 0;

    let mut generated = tga::TgaImage::new(image_width, image_height, 3);
    let white = tga::rgba(255, 255, 255, 255);

    let center_x = image_width / 2;
    let center_y = image_height / 2;
    let content_width = image_width - (padding * 2);
    let content_height = image_height - (padding * 2);
    let content_width = content_width as f64;
    let content_height = content_height as f64;
    let center_x = center_x as f64;
    let center_y = center_y as f64;


//    let model_file = "obj/diablo3_pose/diablo3_pose.obj";
//    let model_file = "D:\\Downloads\\Z3_OBJ\\Z3_OBJ.obj";

//    let model_file = "obj/boggie/eyes.obj";
//    let model_file = "obj/boggie/head.obj";
//    let model_file = "obj/boggie/body.obj";
    let model_file = "obj/african_head/african_head.obj";
    let model = obj::read_obj(model_file).unwrap();
    renderer::render_model(&mut generated, &white, content_width, content_height, center_x, center_y, &model);

    generated.flip_vertically();
    generated.write_tga_file("generated.tga", false).expect("Error while writing output image")

}


