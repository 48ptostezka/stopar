//! QR decoding from camera frames. The app hands over RGBA pixels from a canvas.

/// Converts RGBA pixels to luminance (0 = black, 255 = white).
pub fn rgba_to_luma(rgba: &[u8]) -> Vec<u8> {
    rgba.chunks_exact(4)
        .map(|p| ((p[0] as u32 * 77 + p[1] as u32 * 150 + p[2] as u32 * 29) >> 8) as u8)
        .collect()
}

/// Returns the text of the first readable QR code in the frame.
pub fn decode_luma(width: usize, height: usize, luma: &[u8]) -> Option<String> {
    if width == 0 || height == 0 || luma.len() < width * height {
        return None;
    }
    let mut image = rqrr::PreparedImage::prepare_from_greyscale(width, height, |x, y| luma[y * width + x]);
    image.detect_grids().into_iter().find_map(|grid| grid.decode().ok().map(|(_, text)| text))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Draws a QR code the way a camera would see it: dark modules on white, with a margin.
    fn render(text: &str, module: usize) -> (usize, Vec<u8>) {
        let code = qrcode::QrCode::new(text.as_bytes()).unwrap();
        let n = code.width();
        let size = (n + 8) * module;
        let mut luma = vec![255u8; size * size];
        for y in 0..n {
            for x in 0..n {
                if code[(x, y)] == qrcode::Color::Dark {
                    for dy in 0..module {
                        for dx in 0..module {
                            luma[((y + 4) * module + dy) * size + (x + 4) * module + dx] = 0;
                        }
                    }
                }
            }
        }
        (size, luma)
    }

    #[test]
    fn decodes_a_game_code() {
        let text = crate::payload::encode("galaxy", "7Q4M-2KD9");
        let (size, luma) = render(&text, 6);
        assert_eq!(decode_luma(size, size, &luma).as_deref(), Some(text.as_str()));
    }

    #[test]
    fn blank_frame_has_no_code() {
        assert_eq!(decode_luma(100, 100, &vec![200; 100 * 100]), None);
    }
}
