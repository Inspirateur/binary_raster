use crate::bitline::BitLine;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryRaster(Vec<BitLine>);

impl BinaryRaster {
    pub fn new(width: usize, height: usize) -> Self {
        BinaryRaster(
            (0..height).map(|_| BitLine::new(width)).collect(),
        )
    }

    pub fn from_raster(pixels: &[u8], width: usize) -> Self {
        // .step_by(width) panics if we feed it a value of 0
        if width == 0 {
            return BinaryRaster(Vec::new());
        }
        BinaryRaster(
            (0..pixels.len()).step_by(width).map(|i| BitLine::from_bits(&pixels[i..(i+width)])).collect(),
        )
    }

    /// The amount of allocated usize to represent the widest bitline
    fn max_chunkwidth(&self) -> usize {
        self.0.iter().map(|bit_line| bit_line.chunk_width()).max().unwrap_or(0)
    }

    fn max_chunkwidth_after_shift(&self, amount: u32) -> usize {
        self.0.iter()
            .map(|bit_line| BitLine::chunks_to_fit(bit_line.bits+amount as usize))
            .max().unwrap_or(0)
    }

    fn shifted_right(&self, amount: u32) -> BinaryRaster {
        if amount == 0 {
            return self.clone();
        }
        debug_assert!(amount < usize::BITS);
        BinaryRaster (
            self.0.iter().map(|bitline| bitline.shifted_right(amount)).collect(),
        )
    }

    /// Returns true if other fits within self at given pos, false otherwise
    pub fn can_fit(&self, other: &BinaryRaster, pos: (usize, usize)) -> bool {
        let segment_offset = BitLine::chunks_to_fit(pos.0).max(1)-1;
        let shift_amount = pos.0 as u32 % usize::BITS;
        (segment_offset + other.max_chunkwidth_after_shift(shift_amount) <= self.max_chunkwidth())
        && (pos.1 + other.0.len() < self.0.len())
    }

    fn collision_check(&self, source: &BinaryRaster, segment_offset: usize, line_offset: usize) -> bool {
        for line_i in 0..source.0.len() {
            if self.0[line_i + line_offset].collision_check(&source.0[line_i], segment_offset) {
                return true;
            }
        }
        false
    }

    /// Adds entire source to self at the given position if there's no bit collision and if it fits
    /// Returns Ok(()) if the item was added (no collision), and Err(()) otherwise
    pub fn add_from_checked(&mut self, source: &BinaryRaster, pos: (usize, usize)) -> Result<(), ()> {
        let segment_offset = BitLine::chunks_to_fit(pos.0).max(1)-1;
        let shift_amount = pos.0 as u32 % usize::BITS;
        let source = source.shifted_right(shift_amount);
        if self.collision_check(&source, segment_offset, pos.1) {
            return Err(())
        }
        for line_i in 0..source.0.len() {
            self.0[pos.1 + line_i].add_from(&source.0[line_i], segment_offset);
        }
        Ok(())
    }

    /// Adds entire source to self at the given position without checking from collision, assuming it fits
    pub fn add_from(&mut self, source: &BinaryRaster, pos: (usize, usize)) {
        let segment_offset = BitLine::chunks_to_fit(pos.0).max(1)-1;
        let shift_amount = pos.0 as u32 % usize::BITS;
        let source = source.shifted_right(shift_amount);
        for line_i in 0..source.0.len() {
            self.0[line_i + pos.1].add_from(&source.0[line_i], segment_offset);
        }
    }

    /// Checks if there's any pixel overlap between other and self at given pos
    pub fn collision_check_at(&self, other: &BinaryRaster, pos: (usize, usize)) -> bool {
        if pos.1 >= self.0.len() {
            return false;
        }
        let segment_offset = BitLine::chunks_to_fit(pos.0).max(1)-1;
        let shift_amount = pos.0 as u32 % usize::BITS;
        let other = other.shifted_right(shift_amount);
        let other_height = (other.0.len()+pos.1).min(self.0.len())-pos.1;
        for line_i in 0..other_height {
            if self.0[line_i + pos.1].collision_check(&other.0[line_i], segment_offset) {
                return true;
            }
        }
        false
    }
}


#[cfg(test)]
mod tests {
    use rand::{rngs::ThreadRng, Rng};
    use super::BinaryRaster;
    
    fn random_raster(rng: &mut ThreadRng, width: usize, height: usize, zero_to_one_ratio: u8) -> BinaryRaster {
        let pixels = (0..width*height).map(|_| 1-rng.gen_range(0..=zero_to_one_ratio).min(1)).collect::<Vec<_>>();
        BinaryRaster::from_raster(&pixels, width)
    }
    
    #[test]
    fn test_right_shift() {
        let pixels = vec![
            0, 1, 0, 1, 0,
            0, 1, 0, 1, 0,
            0, 0, 0, 0, 0,
            1, 0, 0, 0, 1,
            0, 1, 1, 1, 0,
        ];
        let raster = BinaryRaster::from_raster(&pixels, 5);
        let shifted_raster = raster.shifted_right(1);
        let shifted_pixels = vec![
            0, 0, 1, 0, 1, 0,
            0, 0, 1, 0, 1, 0,
            0, 0, 0, 0, 0, 0,
            0, 1, 0, 0, 0, 1,
            0, 0, 1, 1, 1, 0,
        ];
        assert_eq!(BinaryRaster::from_raster(&shifted_pixels, 6), shifted_raster);
    }

    #[test]
    fn test_add_no_collision() {
        let mut main_raster = BinaryRaster::from_raster(&vec![
            0, 1, 0, 0, 0,
            1, 1, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 1,
            0, 0, 0, 1, 0,
        ], 5); 
        let added_raster = BinaryRaster::from_raster(&vec![
            1, 1, 1,
            0, 1, 0, 
            0, 1, 0,
        ], 3);
        let res = main_raster.add_from_checked(&added_raster, (2, 1));
        assert_eq!(Ok(()), res);
        let result_raster = BinaryRaster::from_raster(&vec![
            0, 1, 0, 0, 0,
            1, 1, 1, 1, 1,
            0, 0, 0, 1, 0,
            0, 0, 0, 1, 1,
            0, 0, 0, 1, 0,
        ], 5);
        assert_eq!(result_raster, main_raster);
    }

    #[test]
    fn test_add_collision() {
        let mut main_raster = BinaryRaster::from_raster(&vec![
            0, 1, 0, 0, 0,
            1, 1, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 1,
            0, 0, 0, 1, 0,
        ], 5); 
        let added_raster = BinaryRaster::from_raster(&vec![
            1, 1, 1,
            0, 1, 0, 
            0, 1, 0,
        ], 3);
        let res = main_raster.add_from_checked(&added_raster, (1, 1));
        assert_eq!(Err(()), res);
    }

    #[test]
    fn test_collision_at() {
        let raster_a = BinaryRaster::from_raster(&vec![
            0, 1, 0, 0, 0,
            1, 1, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 1,
            0, 0, 0, 1, 0,
        ], 5);
        let raster_b = BinaryRaster::from_raster(&vec![
            1, 1, 1,
            0, 1, 0, 
            0, 1, 0,
        ], 3);
        // there should not be a collision
        assert!(!raster_a.collision_check_at(&raster_b, (3, 0)));
        // there should be a collision
        assert!(raster_a.collision_check_at(&raster_b, (2, 4)));
    }

    #[test]
    fn test_bound_check() {
        let mut rng = rand::thread_rng();
        let main_raster = random_raster(&mut rng, 128, 20, 5);
        let other_raster = random_raster(&mut rng, 20, 2, 0);
        assert!(main_raster.can_fit(&other_raster, (63, 17)));
        assert!(main_raster.can_fit(&other_raster, (107, 9)));
        assert!(!main_raster.can_fit(&other_raster, (110, 0)));
        assert!(!main_raster.can_fit(&other_raster, (10, 18)));
    }
}