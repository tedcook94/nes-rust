pub fn decode_row(low_plane: u8, high_plane: u8) -> [u8; 8] {
    std::array::from_fn(|x| {
        let bit = 7 - x;
        let low = (low_plane >> bit) & 1;
        let high = (high_plane >> bit) & 1;
        (high << 1) | low
    })
}

pub fn decode_tile(bytes: &[u8; 16]) -> [[u8; 8]; 8] {
    std::array::from_fn(|y| decode_row(bytes[y], bytes[y + 8]))
}

#[cfg(test)]
mod tests;
