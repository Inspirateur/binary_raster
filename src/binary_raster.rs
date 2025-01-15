use crate::bitline::BitLine;

#[derive(Clone)]
pub struct BinaryRaster(Vec<BitLine>);

impl BinaryRaster {
    pub fn from_raster(pixels: &[u8], width: usize) -> Self {
        BinaryRaster((0..pixels.len()).step_by(width).map(|i| BitLine::from_bits(&pixels[i..(i+width)])).collect())
    }

    pub fn width(&self) -> usize {
        self.0.iter().map(|bitline| bitline.width()).max().unwrap_or(0)
    }
}


#[cfg(test)]
mod tests {
    use super::BinaryRaster;

    #[test]
    fn test() {
        let pixels = vec![
            0, 1, 0, 1, 0,
            0, 1, 0, 1, 0,
            0, 0, 0, 0, 0,
            1, 0, 0, 0, 1,
            0, 1, 1, 1, 0,
        ];
        let raster = BinaryRaster::from_raster(&pixels, 5);
        assert_eq!(5, raster.width());
    }
}