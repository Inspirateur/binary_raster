# binary_raster
Binary raster crate for efficient pixel-based collision detection. 

Bits are packed in usizes (= u64 on most machines) to make binary rasters both  lightweight and fast.  
Collision checking on a 800x400 raster takes **~1 μs** on a Intel(R) Xeon(R) CPU E5-1650 v3 @ 3.50GHz.
## Example
### Code 
```rust
fn main() {
    // say you got an image you wish to do collision checking against
    let img = todo!()
    let pixels: Vec<u8> = img.grey_scale();
    // build a binary raster like this
    let raster = BinaryRaster::from_raster(&pixels, img.width());
    // and another image you wish to do collision checking with
    let other_img = todo!();
    let other_raster = BinaryRaster::from_raster(&other_img.grey_scale(), other_img.width());
    // fast collision checking with position (25, 40)
    if raster.collision_check_at(&other_raster, (25, 40)) {
        // do stuff
        todo!()
    }
}
```
There are also methods to add rasters to each other, check out [wordcloud-rs](https://crates.io/crates/wordcloud-rs) for a complete usage example.