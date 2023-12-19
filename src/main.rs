use anyhow::Result;
use bytes::Bytes;
use lazy_static::lazy_static;
use libvips::{ops, VipsApp, VipsImage};
use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::request::ResponseData;

lazy_static! {
    static ref BUCKET: &'static str = env!("BUCKET");
    static ref IMAGE: &'static str = "05w9fe2o2wq8kqbzb8bvej1agoq7";
    static ref KEY: &'static str = env!("AWS_ACCESS_KEY_ID");
    static ref SECRET: &'static str = env!("AWS_SECRET_ACCESS_KEY");
    static ref USER_IMAGE: &'static str = "05w9fe2o2wq8kqbzb8bvej1agoq7";
}

pub fn get_bucket() -> Result<Bucket> {
    let credentials = Credentials::new(Some(*KEY), Some(*SECRET), None, None, None)?;
    let region = "ap-southeast-2".parse()?;

    Ok(Bucket::new(*BUCKET, region, credentials)?)
}

pub fn uri_to_bytes(key: &'static str) -> Result<Bytes> {
    let bucket = get_bucket()?;
    let res = bucket.get_object_blocking(key)?;

    Ok(res.bytes().clone())
}

pub fn bytes_to_s3(vec: Vec<u8>) -> Result<ResponseData> {
    let bucket = get_bucket()?;
    let byte_slice: &[u8] = &vec;
    let res = bucket.put_object_blocking("00/test.jpg", byte_slice)?;

    Ok(res)
}

pub fn main() -> Result<()> {
    let post_app = VipsApp::new("PostApp", true)?;
    post_app.concurrency_set(2);

    let source_bytes = uri_to_bytes(*IMAGE)?;
    let logo_bytes = uri_to_bytes(*USER_IMAGE)?;
    let source = VipsImage::new_from_buffer(&source_bytes, "")?;
    let logo = VipsImage::new_from_buffer(&logo_bytes, "")?;
    let result = draw(&source, &logo)?;

    let byte_vec = result.image_write_to_buffer(&".jpg")?;
    let res = bytes_to_s3(byte_vec)?;
    println!("status: {}", res.status_code());

    Ok(())
}

fn draw(original: &VipsImage, user_image: &VipsImage) -> Result<VipsImage> {
    let w: f64 = original.get_width() as f64;
    let h: f64 = original.get_height() as f64;
    let scale: f64 = (w / 4.0) / user_image.get_width() as f64;
    let resized_user_image = ops::resize(&user_image, scale)?;
    let banner_h = resized_user_image.get_height() + 20;

    let banner_frame = VipsImage::image_new_matrix(w as i32, banner_h)?;
    let mut ink_vec = hex_to_rgb("#ffffff").unwrap();
    let ink: &mut [f64] = &mut ink_vec;
    let _ = ops::draw_flood(&banner_frame, ink, 0, 0)?;
    let banner = ops::insert(&banner_frame, &resized_user_image, 10, 10)?;

    let source_frame = VipsImage::image_new_matrix(w as i32, h as i32 + banner_h)?;
    let source = ops::insert(&source_frame, original, 0, 0)?;
    let res = ops::insert(&source, &banner, 0, h as i32)?;

    let mut user_ink_vec = hex_to_rgb("#0055ff").unwrap();
    user_ink_vec.push(100.0);
    
    let user_ink: &mut [f64] = &mut user_ink_vec;
    ops::draw_line(&res, user_ink, 0, h as i32, w as i32, h as i32)?;

    Ok(res)
}

fn hex_to_rgb(hex: &str) -> Result<Vec<f64>, String> {
    if hex.starts_with('#') && hex.len() == 7 {
        let r = u8::from_str_radix(&hex[1..3], 16);
        let g = u8::from_str_radix(&hex[3..5], 16);
        let b = u8::from_str_radix(&hex[5..7], 16);

        match (r, g, b) {
            (Ok(r), Ok(g), Ok(b)) => Ok(vec![r as f64, g as f64, b as f64]),
            _ => Err("Invalid hex color".to_string()),
        }
    } else {
        Err("Invalid hex color format".to_string())
    }
}
