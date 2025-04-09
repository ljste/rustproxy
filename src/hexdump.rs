pub fn format_hex_dump(data: &[u8], prefix: &str) -> String {
    let mut result = String::new();
    let mut offset = 0;
    
    while offset < data.len() {
        let chunk_size = std::cmp::min(16, data.len() - offset);
        let chunk = &data[offset..offset + chunk_size];
        
        result.push_str(prefix);
        
        result.push_str(&format!("{:04x}  ", offset));
        
        for (i, &byte) in chunk.iter().enumerate() {
            result.push_str(&format!("{:02x} ", byte));
            if i == 7 {
                result.push_str(" ");
            }
        }
        
        if chunk_size < 16 {
            for _ in 0..(16 - chunk_size) {
                result.push_str("   ");
            }
            if chunk_size <= 8 {
                result.push_str(" ");
            }
        }
        
        result.push_str(" |");
        for &byte in chunk {
            if byte >= 32 && byte <= 126 {
                result.push(byte as char);
            } else {
                result.push('.');
            }
        }
        result.push_str("|\n");
        
        offset += chunk_size;
    }
    
    result
}