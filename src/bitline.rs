use core::fmt::Debug;
const BIT_1: &str = "██";
const BIT_0: &str = "  ";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BitLine {
    data: Vec<usize>,
    pub(crate) bits: usize,
}

impl BitLine {
    /// Turns a "continuous" position i into a "chunked" position i
    /// returning the index of the u64 and the position of the bit inside that u64
    pub fn chunked(i: usize) -> (usize, u32) {
        (
            i / usize::BITS as usize,
            i as u32 % usize::BITS
        )
    }

    /// How many u64 are needed to store this amount of bits ? 
    pub fn chunks_to_fit(bits: usize) -> usize {
        bits / usize::BITS as usize + if bits % usize::BITS as usize == 0 { 0 } else { 1 }
    }

    pub fn new(bits: usize) -> Self {
        Self { data: vec![0; BitLine::chunks_to_fit(bits)], bits }
    }

    pub fn from_bits(bits: &[u8]) -> Self {
        let chunkslen = BitLine::chunks_to_fit(bits.len());
        let mut data = vec![0; chunkslen as usize];
        let mut chunk_i = 0;
        let mut bit_i = 0;
        for &bit in bits {
            if bit != 0 {
                data[chunk_i] |= 1 << bit_i;
            }
            bit_i += 1;
            if bit_i >= usize::BITS {
                bit_i = 0;
                chunk_i += 1;
            }
        }
        Self { data, bits: bits.len() }
    }

    pub fn to_bits(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(self.bits);
        for (seg_i, &segment) in self.data.iter().enumerate() {
            for bit_i in 0..usize::BITS as usize {
                if seg_i*usize::BITS as usize + bit_i >= self.bits {
                    return res;
                }
                res.push(((segment >> bit_i) & 1) as u8);
            }
        }
        res
    }

    /// The position of the first bit with a value of 1 in the line
    pub fn start(&self) -> Option<usize> {
        for (i, segment) in self.data.iter().enumerate() {
            let trailing_zeros = segment.trailing_zeros();
            if trailing_zeros < usize::BITS {
                return Some(i*usize::BITS as usize + trailing_zeros as usize)
            }
        }
        None
    }

    /// The position of the last bit with a value of 1 in the line
    pub fn end(&self) -> Option<usize> {
        for (i, segment) in self.data.iter().enumerate().rev() {
            let leading_zeros = segment.leading_zeros();
            if leading_zeros < usize::BITS {
                return Some((i+1)*usize::BITS as usize - leading_zeros as usize - 1)
            }
        }
        None
    }

    /// end - start + 1 or 0 if the line is empty
    pub fn width(&self) -> usize {
        let Some(end) = self.end() else {
            return 0;
        };
        end - self.start().unwrap() + 1
    }

    /// The amount of usize that are used to represent the bitline
    pub fn chunk_width(&self) -> usize {
        self.data.len()
    }

    /// Shifts the bits of the bitline to the right, assumes the shifting amount is less than usize::BITS (32 or 64)
    pub fn shifted_right(&self, amount: u32) -> BitLine {
        if amount == 0 {
            return self.clone();
        }
        debug_assert!(amount < usize::BITS);
        let mut res = Vec::with_capacity(self.data.len()+1);
        self.data.clone_into(&mut res);
        for i in (1..=res.len()).rev() {
            res[i-1] <<= amount;
            let spill = self.data[i-1] >> (usize::BITS-amount);
            if spill == 0 {
                continue;
            }
            if i < res.len() {
                res[i] |= spill;
            } else {
                res.push(spill);
            }
        }
        BitLine {
            data: res,
            bits: self.bits + amount as usize,
        }
    }

    /// Checks if other have 1 bit in common with self at the given offset
    pub fn collision_check(&self, other: &BitLine, segment_offset: usize) -> bool {
        if segment_offset >= self.data.len() {
            return false;
        }
        let other_len = (other.data.len()+segment_offset).min(self.data.len())-segment_offset;
        for i in 0..other_len {
            if self.data[i+segment_offset] & other.data[i] != 0 {
                return true;
            }
        }
        false
    }

    /// Add the entire source to self at the given offset, assuming it fits
    pub fn add_from(&mut self, source: &BitLine, segment_offset: usize) {
        debug_assert!(source.data.len()+segment_offset <= self.data.len());
        for i in 0..source.data.len() {
            self.data[i+segment_offset] |= source.data[i];
        }
    }

    /// Gets a String display of the bitline at the desired resolution, with "■" for 1 and " " for 0
    /// A resolution of 1 displays every bit, 2 displays 1/2 bits, etc.
    pub fn get_display(&self, resolution: u32) -> String {
        // a resolution of 0 would panic because of .step_by(resolution)
        if resolution == 0 {
            return String::new();
        }
        self.to_bits()
            .into_iter()
            .step_by(resolution as usize)
            .map(|bit| if bit == 1 { BIT_1 } else { BIT_0 })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{BitLine, BIT_0, BIT_1};
    use rand::Rng;

    #[test]
    fn test_roundtrip() {
        let mut rng = rand::thread_rng();
        let truth = (0..100).map(|_| rng.gen_range(0..=1)).collect::<Vec<_>>();
        let bitline = BitLine::from_bits(&truth);
        assert_eq!(truth, bitline.to_bits());
    }

    fn collision_check(a: &[u8], b: &[u8]) -> bool {
        BitLine::from_bits(a).collision_check(&BitLine::from_bits(b), 0)
    }

    #[test]
    fn test_collision() {
        let should_be_false = collision_check(
            &vec![0, 1, 1, 0, 1, 0],
            &vec![1, 0, 0, 0, 0, 1],
        );
        assert!(!should_be_false);
        let should_be_true = collision_check(
            &vec![0, 1, 1, 0, 1, 0],
            &vec![1, 0, 0, 0, 1, 0],
        );
        assert!(should_be_true);
    }

    #[test]
    fn test_shift() {
        let shift_amount = 5;
        let mut rng = rand::thread_rng();
        let truth = (0..100).map(|_| rng.gen_range(0..=1)).collect::<Vec<_>>();
        let mut shifted_truth = vec![0; truth.len()+shift_amount];
        shifted_truth[shift_amount..].copy_from_slice(&truth);
        let bitline = BitLine::from_bits(&truth);
        let shifted_bitline = bitline.shifted_right(shift_amount as u32);
        assert_eq!(shifted_truth, shifted_bitline.to_bits());
    }

    #[test]
    fn test_start() {
        let bitline = BitLine::from_bits(&vec![0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0]);
        assert_eq!(Some(2), bitline.start());
    }

    #[test]
    fn test_end() {
        let bitline = BitLine::from_bits(&vec![0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0]);
        assert_eq!(Some(7), bitline.end());
    }

    #[test]
    fn test_width() {
        let bitline = BitLine::from_bits(&vec![0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0]);
        assert_eq!(6, bitline.width());
    }

    #[test]
    fn test_display() {
        let mut rng = rand::thread_rng();
        let bits = (0..100).map(|_| rng.gen_range(0..=1)).collect::<Vec<_>>();
        let bitline = BitLine::from_bits(&bits);
        let truth = bits.into_iter().map(|bit| if bit == 1 { BIT_1 } else { BIT_0 } ).collect::<String>();
        assert_eq!(truth, bitline.get_display(1));
        // test half resolution
        let bitline = BitLine::from_bits(&vec![0, 1, 1, 1, 0, 1, 0, 0, 1]);
        assert_eq!(vec![BIT_0, BIT_1, BIT_0, BIT_0, BIT_1].into_iter().collect::<String>(), bitline.get_display(2));
    }
}