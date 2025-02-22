const HEX_CHAR_LOOKUP: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
];

pub fn as_hex(array: &[u8]) -> String {
    let mut hex_string = String::new();
    for byte in array {
        hex_string.push(HEX_CHAR_LOOKUP[(byte >> 4) as usize]);
        hex_string.push(HEX_CHAR_LOOKUP[(byte & 0xF) as usize]);
    }
    hex_string
}
