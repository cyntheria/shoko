pub fn compress(data: &[u8], clevel: u8) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }

    let mut compressed = Vec::new();
    let mut i = 0;

    let threshold = match clevel {
        1..=3 => 4,
        4..=6 => 3,
        7..=9 => 2,
        _ => 3,
    };

    while i < data.len() {
        let mut run_len = 1;
        while i + run_len < data.len() && 
              data[i + run_len] == data[i] && 
              run_len < 255 {
            run_len += 1;
        }

        if run_len >= threshold {
            compressed.push(0x00); 
            compressed.push(run_len as u8);
            compressed.push(data[i]);
            i += run_len;
        } else {
            let mut literal_end = i;
            while literal_end < data.len() && 
                  (literal_end + 1 >= data.len() || data[literal_end] != data[literal_end + 1]) &&
                  (literal_end - i) < 254 {
                literal_end += 1;
            }
            
            let lit_len = literal_end - i;
            if lit_len > 0 {
                compressed.push(0x01);
                compressed.push(lit_len as u8);
                compressed.extend_from_slice(&data[i..literal_end]);
                i = literal_end;
            } else {
                compressed.push(0x01);
                compressed.push(1);
                compressed.push(data[i]);
                i += 1;
            }
        }
    }
    compressed
}
