use super::*;

#[test]
fn row_of_zeros_is_color_zero() {
    assert_eq!(decode_row(0x00, 0x00), [0; 8]);
}

#[test]
fn low_plane_sets_bit_zero() {
    assert_eq!(decode_row(0xFF, 0x00), [1; 8]);
}

#[test]
fn high_plane_sets_bit_one() {
    assert_eq!(decode_row(0x00, 0xFF), [2; 8]);
}

#[test]
fn both_planes_make_color_three() {
    assert_eq!(decode_row(0xFF, 0xFF), [3; 8]);
}

#[test]
fn bit_seven_is_leftmost_pixel() {
    assert_eq!(decode_row(0b1000_0000, 0x00), [1, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(decode_row(0b0000_0001, 0x00), [0, 0, 0, 0, 0, 0, 0, 1]);
}

#[test]
fn mixed_planes_combine_per_pixel() {
    assert_eq!(
        decode_row(0b0101_0101, 0b0011_0011),
        [0, 1, 2, 3, 0, 1, 2, 3]
    );
}

#[test]
fn tile_pairs_row_n_with_byte_n_plus_8() {
    // Row 0 uses bytes 0 and 8; row 7 uses bytes 7 and 15.
    let mut bytes = [0u8; 16];
    bytes[0] = 0xFF;
    bytes[15] = 0xFF;
    let tile = decode_tile(&bytes);

    assert_eq!(tile[0], [1; 8]);
    assert_eq!(tile[7], [2; 8]);
    assert_eq!(tile[1], [0; 8]);
}

#[test]
fn decodes_known_tile() {
    // The "½" tile from the NESdev wiki's PPU pattern tables page.
    let bytes = [
        0x41, 0xC2, 0x44, 0x48, 0x10, 0x20, 0x40, 0x80, // low plane
        0x01, 0x02, 0x04, 0x08, 0x16, 0x21, 0x42, 0x87, // high plane
    ];
    let tile = decode_tile(&bytes);

    assert_eq!(tile[0], [0, 1, 0, 0, 0, 0, 0, 3]);
    assert_eq!(tile[1], [1, 1, 0, 0, 0, 0, 3, 0]);
    assert_eq!(tile[4], [0, 0, 0, 3, 0, 2, 2, 0]);
    assert_eq!(tile[7], [3, 0, 0, 0, 0, 2, 2, 2]);
}
