use nokhwa::{Camera, utils::{CameraIndex, RequestedFormat, RequestedFormatType}};
use rqrr::PreparedImage;

pub fn scan_qr() -> Option<String> {
    let mut camera = Camera::new(
        CameraIndex::Index(0),
        RequestedFormat::new::<nokhwa::pixel_format::RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate)
    ).expect("You need to connect a camera to use this.");

    camera.open_stream().unwrap();

    loop {
        let frame = camera.frame().unwrap();
        let decoded = frame.decode_image::<nokhwa::pixel_format::RgbFormat>().unwrap();
        let luma = image::DynamicImage::ImageRgb8(decoded).to_luma8();
        let mut prepared = PreparedImage::prepare(luma);
        let grids = prepared.detect_grids();

        if let Some(grid) = grids.first() {
            if let Ok((_, content)) = grid.decode() {
                return Some(content);
            }
        }
    }
}