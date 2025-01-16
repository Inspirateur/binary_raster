use crate::bitline::BitLine;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryRaster {
    data: Vec<BitLine>, 
    // tracks the amount of right shifting (only a positive amount is planned for now)
    // TODO: this needs to be accounted for when positionning the raster against other
    x_offset: u32,
}

impl BinaryRaster {
    pub fn from_raster(pixels: &[u8], width: usize) -> Self {
        BinaryRaster {
            data: (0..pixels.len()).step_by(width).map(|i| BitLine::from_bits(&pixels[i..(i+width)])).collect(),
            x_offset: 0
        }
    }

    /// The amount of allocated usize to represent the widest bitline
    fn max_chunkwidth(&self) -> usize {
        self.data.iter().map(|bit_line| bit_line.chunk_width()).max().unwrap_or(0)
    }

    fn width(&self) -> usize {
        let Some(min_start) = self.data.iter().filter_map(|bitline| bitline.start()).min() else {
            return 0;
        };
        let max_end = self.data.iter().filter_map(|bitline| bitline.end()).max().unwrap();
        max_end - min_start + 1
    }

    fn shifted_right(&self, amount: u32) -> BinaryRaster {
        if amount == 0 {
            return self.clone();
        }
        debug_assert!(amount < usize::BITS);
        BinaryRaster { 
            data: self.data.iter().map(|bitline| bitline.shifted_right(amount)).collect(),
            x_offset: self.x_offset + amount
        }
    }

    fn collision_check(&self, source: &BinaryRaster, segment_offset: usize, line_offset: usize) -> bool {
        for line_i in 0..source.data.len() {
            if self.data[line_i + line_offset].collision_check(&source.data[line_i], segment_offset) {
                return true;
            }
        }
        false
    }

    /// Adds entire source to self at the given position if there's no bit collision, assuming it fits
    /// Returns Ok(()) if the item was added (no collision), and Err(()) otherwise
    pub fn add_from_checked(&mut self, source: &BinaryRaster, pos: (usize, usize)) -> Result<(), ()> {
        let segment_offset = (BitLine::chunks_to_fit(pos.0)-1).max(0);
        debug_assert!(segment_offset + source.max_chunkwidth() <= self.max_chunkwidth());
        debug_assert!(pos.1 + source.data.len() <= self.data.len());
        let shift_amount = pos.0 as u32 % usize::BITS;
        let source = source.shifted_right(shift_amount);
        if self.collision_check(&source, segment_offset, pos.1) {
            return Err(())
        }
        for line_i in 0..source.data.len() {
            self.data[pos.1 + line_i].add_from(&source.data[line_i], segment_offset);
        }
        Ok(())
    }

    /// Adds entire source to self at the given position without checking from collision, assuming it fits
    pub fn add_from(&mut self, source: &BinaryRaster, pos: (usize, usize)) {
        let segment_offset = (BitLine::chunks_to_fit(pos.0)-1).max(0);
        debug_assert!(segment_offset + source.max_chunkwidth() <= self.max_chunkwidth());
        debug_assert!(pos.1 + source.data.len() <= self.data.len());
        let shift_amount = pos.0 as u32 % usize::BITS;
        let source = source.shifted_right(shift_amount);
        for line_i in 0..source.data.len() {
            self.data[line_i + pos.1].add_from(&source.data[line_i], segment_offset);
        }
    }

    /// a naive blur that simply extends every pixel by 1 in all directions; useful to check for near collisions
    pub fn blur(&mut self) {
        if self.data.len() == 0 {
            return;
        }
        let clone = self.clone();
        // insert empty bitlines at start and end to make room for vertical bluring
        self.data.insert(0, self.data[0].clone());
        self.data.push(self.data[self.data.len()-1].clone());
        // shift right twice to make room for the vertical bluring
        *self = self.shifted_right(2);
        // add itself at various offset for the 4 directions
        self.add_from(&clone, (1, 0));
        self.add_from(&clone, (1, 2));
        self.add_from(&clone, (0, 1));
        self.add_from(&clone, (1, 1));
    }

    /// Checks if there's any pixel overlap between other and self at given pos
    pub fn collision_check_at(&self, other: &BinaryRaster, pos: (usize, usize)) -> bool {
        if pos.1 >= self.data.len() {
            return false;
        }
        let segment_offset = (BitLine::chunks_to_fit(pos.0)-1).max(0);
        let shift_amount = pos.0 as u32 % usize::BITS;
        let other = other.shifted_right(shift_amount);
        let other_height = (other.data.len()+pos.1).min(self.data.len())-pos.1;
        for line_i in 0..other_height {
            if self.data[line_i + pos.1].collision_check(&other.data[line_i], segment_offset) {
                return true;
            }
        }
        false
    }
}


#[cfg(test)]
mod tests {
    use super::BinaryRaster;

    #[test]
    fn test_width() {
        let pixels = vec![
            0, 1, 0, 0, 0,
            0, 1, 1, 0, 0,
            0, 0, 1, 1, 0,
            0, 0, 0, 1, 1,
            0, 0, 1, 0, 0,
        ];
        let raster = BinaryRaster::from_raster(&pixels, 5);
        assert_eq!(4, raster.width());
    }

    #[test]
    fn test_empty_width() {
        let pixels = vec![
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let raster = BinaryRaster::from_raster(&pixels, 5);
        assert_eq!(0, raster.width());
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
    fn test_blur() {
        let mut raster = BinaryRaster::from_raster(&vec![
            0, 1, 0, 0, 
            0, 1, 0, 0,
            0, 0, 0, 0, 
            0, 0, 0, 1,
        ], 4);
        let blured_raster = BinaryRaster::from_raster(&vec![
            0, 0, 1, 0, 0, 0,
            0, 1, 1, 1, 0, 0, 
            0, 1, 1, 1, 0, 0, 
            0, 0, 1, 0, 1, 0,
            0, 0, 0, 1, 1, 1,
            0, 0, 0, 0, 1, 0,
        ], 6);
        raster.blur();
        assert_eq!(blured_raster, raster);
    }
}