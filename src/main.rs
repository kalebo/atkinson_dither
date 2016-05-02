extern crate image;

use std::path::Path;
use std::fs::File;
use std::env;

use image::{
    GenericImage,
    //ImageBuffer,
    Pixel,
    Rgba
};

fn main() {

    let args_vec: Vec<String> = env::args().collect();
    if args_vec.len() != 2 {
        println!("Path to an image must be specified!");
        std::process::exit(1);
    }

    let mut img = image::open(&Path::new(&args_vec[1])).expect("Opening file failed!");
    println!("Loaded '{}' with dimensions {:?} and color type {:?}", args_vec[1], img.dimensions(), img.color());

    atkinson(&mut img);
    assert!(check_two_tone(&img), "There are values other than 0 and 255!");

    let ref mut fout = File::create(&Path::new
    ("test_out.png")).expect("Failed creating file!");
    let _ = img.save(fout, image::PNG).expect("Saving failed!");

}


/// Applies an Atkinson dither to a mutable DynamicImage
fn atkinson(img: &mut image::DynamicImage) {

    let (width, height) = img.dimensions();

    // We could also impliment this using img.raw_pixels...

    // We scan first by row so we don't end up steping on previously written pixels
    for y in 0..(height) {
        for x in 0..(width) {

            let px = img.get_pixel(x, y);

            // Push the value of a pixel to either black or white
            let newpx = px.map(|v| if v < 128 {0} else {255});
            img.put_pixel(x, y, newpx);

            let errpx = calc_errpx(&px, &newpx);

            let mut near = vec![(x+1, y), (x+2, y),
                        (x, y+1), (x+1, y+1), (x, y+2)];

            // protects against underflowing x: u32
            if x != 0 {
                near.push((x-1, y+1));
            }

            for (nx, ny) in near.into_iter() {

                if img.in_bounds(nx, ny) {

                    let corr = add_pixels(&img.get_pixel(nx, ny), &errpx);
                    img.put_pixel(nx, ny, corr);

                }
            }
        }
    }
}

/// Calculates the difference or error between the original pixel and
/// the the new color.
fn calc_errpx(originalpx: &Rgba<u8>, newpx: &Rgba<u8>) -> Rgba<u8> {

    let mut result = originalpx.data;
    let new = newpx.data;

    for i in 0..4 {
        // a bit shift by three is the same as dividing by 8
        result[i] = (result[i].saturating_sub(new[i])) >> 3; 
    }

    Rgba::<u8> {data: result}
}

/// Adds the value of two pixels together
/// Really I think that the color types should impliment std::ops::Add in the image crate. Maybe I'll submit a pull request...
fn add_pixels(a: &Rgba<u8>, b: &Rgba<u8>) -> Rgba<u8> {

    let mut result = a.data;
    let diff = b.data;

    for i in 0..4 {
        // Once again we need to use saturating_add so we don't overflow the u8 primitive
        result[i] = result[i].saturating_add(diff[i]);
    }

    Rgba::<u8> {data: result}
}

/// Checks the image to ensure that we have produced a valid dithered image
fn check_two_tone(img: &image::DynamicImage) -> bool {
    let (width, height) = img.dimensions();

    for y in 0..(height) {
        for x in 0..(width) {

            let px = img.get_pixel(x, y);
            for i in 0..4 {

                if px.data[i] != 0 && px.data[i] != 255 {
                    return false;
                }

            }
        }
    }
    true
}

