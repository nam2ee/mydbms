
pub fn read_varint(data: &[u8]) -> Result<(u64, usize), Box<dyn std::error::Error>> {
    let mut result = 0u64;
    let mut bytes_read = 0;
    
    for i in 0..9 { 
        if i >= data.len() {
            return Err("Unexpected end of data".into());
        }
        
        let byte = data[i];
        bytes_read += 1;
        
        if i < 8 {
            // 처음 8바이트: 하위 7비트만 사용
            result = (result << 7) | ((byte & 0x7f) as u64);
            if byte & 0x80 == 0 {
                // MSB=0이면 끝
                break;
            }
        } else {
            // 9번째 바이트: 모든 비트 사용
            result = (result << 8) | (byte as u64);
            break;
        }
    }
    
    Ok((result, bytes_read)) // (값, 읽은 바이트 수)
}