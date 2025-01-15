#[derive(Clone)]
pub(crate) struct BitLine {
    data: Vec<usize>,
    bits: usize
}

impl BitLine {
    pub fn from_bits(bits: &[u8]) -> Self {
        let bitslen = bits.len() as u32;
        let chunkslen = bitslen / usize::BITS + if bitslen % usize::BITS == 0 { 0 } else { 1 };
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
        Self { data, bits: bitslen as usize }
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
    fn start(&self) -> Option<usize> {
        for (i, segment) in self.data.iter().enumerate() {
            let leading_zeros = segment.leading_zeros();
            if leading_zeros < usize::BITS {
                return Some(i*usize::BITS as usize + leading_zeros as usize)
            }
        }
        None
    }

    /// The position of the last bit with a value of 1 in the line
    fn end(&self) -> Option<usize> {
        for (i, segment) in self.data.iter().enumerate().rev() {
            let trailing_zeros = segment.trailing_zeros();
            if trailing_zeros < usize::BITS {
                return Some((i+1)*usize::BITS as usize - trailing_zeros as usize - 1)
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

    /// Checks if self have 1 bit in common with other
    pub fn collision_check(&self, other: &BitLine) -> bool {
        for i in 0..self.data.len().min(other.data.len()) {
            if self.data[i] & other.data[i] != 0 {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::BitLine;
    use rand::Rng;

    #[test]
    fn test_roundtrip() {
        let mut rng = rand::thread_rng();
        let truth = (0..100).map(|_| rng.gen_range(0..=1)).collect::<Vec<_>>();
        let bitline = BitLine::from_bits(&truth);
        assert_eq!(truth, bitline.to_bits());
    }

    fn collision_check(a: &[u8], b: &[u8]) -> bool {
        BitLine::from_bits(a).collision_check(&BitLine::from_bits(b))
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
    fn test_width() {
        let bitline = BitLine::from_bits(&vec![0, 1, 0, 0, 1, 0, 1, 1, 0, 0, 0]);
        assert_eq!(7, bitline.width());
    }
}