use std::{collections::HashMap, error::Error};
use crate::util::read_varint;
use regex::Regex;





pub struct Cell{}

impl Cell{

    pub fn read_cell_pointer_array(page_start: &[u8], cell_count: u16) -> Vec<u16> {

        let mut pointers = Vec::new();
        let start_offset = 8; 
        
        for i in 0..cell_count {
            let pos = start_offset + (i * 2) as usize;
            let pointer = u16::from_be_bytes([page_start[pos], page_start[pos + 1]]);
            pointers.push(pointer);
        }

        pointers
    }


    pub fn parse_cell(data: &[u8], offset: usize) -> Result<Vec<String>, Box<dyn Error>> {
        let mut pos = offset;

        let (_, len1) = read_varint(&data[pos..])?;
        pos += len1;


        let (_, len2) = read_varint(&data[pos..])?;
        pos += len2;

        let (header_size, len3) = read_varint(&data[pos..])?;
        let header_start = pos;
        pos += len3;

    
        let mut serial_types = Vec::new();
        while pos - header_start < header_size as usize {
            let (serial_type, len) = read_varint(&data[pos..])?;
            serial_types.push(serial_type);
            pos += len;
        }

    
        let mut values = Vec::new();
        for &serial_type in &serial_types {
            let value = SerialCode::read_value_by_serial_type(&data[pos..], serial_type)?;
            pos += SerialCode::value_size(serial_type);
            values.push(value);
        }

        Ok(values) // [type, name, tbl_name, rootpage, sql]
    }

    pub fn parse_cell_as_map(
        data: &[u8], 
        offset: usize, 
        column_names: &[String]
    ) -> Result<HashMap<String, String>, Box<dyn Error>> {
        let values = Self::parse_cell(data, offset)?;
        
        if values.len() != column_names.len() {
            return Err(format!(
                "Column count mismatch: expected {}, got {}", 
                column_names.len(), 
                values.len()
            ).into());
        }

        let mut row = HashMap::new();
        for (col_name, value) in column_names.iter().zip(values.iter()) {
            row.insert(col_name.clone(), value.clone());
        }

        Ok(row)
    }


    pub fn parse_create_table(sql: &str) -> Result<Vec<String>, Box<dyn Error>> {
        // (?s)는 dotall 모드 - '.'가 newline도 매치하도록 함
        // *? 는 lazy matching - 가능한 최소로 매치
        let re = Regex::new(r"(?s)CREATE TABLE \w+\s*\((.*?)\)")?;

        if let Some(caps) = re.captures(sql) {
            let columns_str = &caps[1];

            // 쉼표로 분리하되, 괄호 안의 쉼표는 무시 (예: CHECK 제약조건)
            let columns: Vec<String> = columns_str
                .split(',')
                .filter_map(|col| {
                    let trimmed = col.trim();

                    // 빈 문자열이나 제약조건은 스킵
                    if trimmed.is_empty() {
                        return None;
                    }

                    // 테이블 제약조건 스킵 (PRIMARY KEY, FOREIGN KEY, CHECK 등)
                    let upper = trimmed.to_uppercase();
                    if upper.starts_with("PRIMARY KEY") 
                       || upper.starts_with("FOREIGN KEY")
                       || upper.starts_with("UNIQUE")
                       || upper.starts_with("CHECK")
                       || upper.starts_with("CONSTRAINT") {
                        return None;
                    }

                    // 첫 번째 단어만 추출 (컬럼 이름)
                    trimmed
                        .split_whitespace()
                        .next()
                        .map(|s| s.to_string())
                })
                .collect();
            
            Ok(columns)
        } else {
            Err("Failed to parse CREATE TABLE".into())
        }
    }

    

}

struct SerialCode {}

impl SerialCode{
    fn value_size(serial_type: u64) -> usize {
        match serial_type {
            0 => 0,
            1 => 1,
            2 => 2,
            3 => 3,
            4 => 4,
            5 => 6,
            6 => 8,
            7 => 8,
            8 | 9 => 0,
            n if n >= 12 && n % 2 == 0 => ((n - 12) / 2) as usize, // BLOB
            n if n >= 13 && n % 2 == 1 => ((n - 13) / 2) as usize, // TEXT
            _ => 0
        }
    }

    fn read_value_by_serial_type(data: &[u8], serial_type: u64) -> Result<String, Box<dyn std::error::Error>> {
    match serial_type {
        0 => Ok("NULL".to_string()),
        
        1 => {
            // 8비트 signed integer
            if data.is_empty() { return Err("No data for integer".into()); }
            Ok(format!("{}", data[0] as i8))
        },
        
        2 => {
            // 16비트 signed integer (big-endian)
            if data.len() < 2 { return Err("Not enough data for i16".into()); }
            let value = i16::from_be_bytes([data[0], data[1]]);
            Ok(format!("{}", value))
        },
        
        3 => {
            // 24비트 signed integer (big-endian)
            if data.len() < 3 { return Err("Not enough data for i24".into()); }
            let mut bytes = [0u8; 4];
            bytes[1..4].copy_from_slice(&data[0..3]);
            let value = i32::from_be_bytes(bytes);
            // 24비트이므로 부호 확장 필요
            let value = if value & 0x800000 != 0 {
                value | 0xff000000u32 as i32 
            } else {
                value
            };
            Ok(format!("{}", value))
        },
        
        4 => {

            if data.len() < 4 { return Err("Not enough data for i32".into()); }
            let value = i32::from_be_bytes([data[0], data[1], data[2], data[3]]);
            Ok(format!("{}", value))
        },
        
        8 => Ok("0".to_string()),    // 상수 0
        9 => Ok("1".to_string()),    // 상수 1
        
        // TEXT: N ≥ 13이고 홀수
        n if n >= 13 && n % 2 == 1 => {
            let length = ((n - 13) / 2) as usize;
            if data.len() < length { 
                return Err(format!("Not enough data for text: need {}, have {}", length, data.len()).into()); 
            }
            let text_bytes = &data[0..length];
            match String::from_utf8(text_bytes.to_vec()) {
                Ok(s) => Ok(s),
                Err(_) => Ok(format!("Invalid UTF-8: {:?}", text_bytes))
            }
        },
        
        // BLOB: N ≥ 12이고 짝수
        n if n >= 12 && n % 2 == 0 => {
            let length = ((n - 12) / 2) as usize;
            if data.len() < length { 
                return Err("Not enough data for blob".into()); 
            }
            Ok(format!("BLOB({} bytes): {:?}", length, &data[0..length]))
        },
        
        _ => Ok(format!("Unknown serial type: {}", serial_type))
    }
}
    
}




