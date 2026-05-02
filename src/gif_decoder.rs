use gif::DecodeOptions;
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};
use std::fs::File;


pub fn decode_gif_frames(path: &str) -> Vec<Image> {
    let file = File::open(path).expect("Failed to open GIF");
    let mut decoder = DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::RGBA);
    let mut decoder = decoder.read_info(file).unwrap();

    let width = decoder.width() as u32;
    let height = decoder.height() as u32;
    let mut frames = Vec::new();

    while let Some(frame) = decoder.read_next_frame().unwrap() {
        let buffer = frame.buffer.to_vec();
        let pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
            &buffer,
            width,
            height,
        );
        frames.push(Image::from_rgba8(pixel_buffer));
    }

    return frames;
}