pub fn decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    if data.is_empty() {
        return Ok(Vec::new());
    }

    let mut decompressed = Vec::new();
    let mut i = 0;

    while i < data.len() {
        let control_byte = data[i];
        i += 1;

        match control_byte {
            0x00 => {
                if i + 1 >= data.len() {
                    return Err("Malformed Shoko stream: unexpected end of run".to_string());
                }
                let run_len = data[i] as usize;
                let value = data[i + 1];
                
                for _ in 0..run_len {
                    decompressed.push(value);
                }
                
                i += 2;
            }
            0x01 => {
                if i >= data.len() {
                    return Err("Malformed Shoko stream: missing literal length".to_string());
                }
                let lit_len = data[i] as usize;
                i += 1;

                if i + lit_len > data.len() {
                    return Err("Malformed Shoko stream: literal length exceeds data".to_string());
                }

                decompressed.extend_from_slice(&data[i..i + lit_len]);
                i += lit_len;
            }
            _ => {
                return Err(format!("Invalid Shoko control byte: {:#04x}", control_byte));
            }
        }
    }

    Ok(decompressed)
}
