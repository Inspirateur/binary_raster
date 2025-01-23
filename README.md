# binary_raster
Binary raster crate for efficient pixel-based collision detection. 

Bits are packed in usizes (= u64 on most machines) to make binary rasters both  lightweight and fast.  
Collision checking on a 800x400 raster takes **~1 μs** on a Intel(R) Xeon(R) CPU E5-1650 v3 @ 3.50GHz.
## Example
### Code 
```rust
fn main() {
    // say you got an image you wish to do collision checking against
    let img_a = todo!()
    let pixels_a: Vec<u8> = img_a.grey_scale();
    // build a binary raster like this
    let raster_a = BinaryRaster::from_raster(&pixels_a, img_a.width());
    // and another image you wish to do collision checking with
    let img_b = todo!();
    let pixels_b: Vec<u8> = img_b.grey_scale();
    let raster_b = BinaryRaster::from_raster(&pixels_b, img_b.width());
    // collision checking of raster_b
    // positionned at (25, 40) relatively to raster_a
    if raster_a.collision_check_at(&raster_b, (25, 40)) {
        // There is a collision
        todo!()
    }
}
```
There are also methods to add rasters to each other, check out [wordcloud-rs](https://crates.io/crates/wordcloud-rs) for a complete usage example.