use std::collections::HashMap;


pub fn extract_columns(commands: &Vec<&str>) -> Vec<String> {
    // SELECT 다음부터 FROM 전까지
    let mut columns = Vec::new();
    let mut in_select = false;

    for &word in commands {
        let upper = word;

        if upper == "select" {
            in_select = true;
            continue;
        }

        if upper == "from" || upper == "where" || upper == "order" {
            break;
        }

        if in_select {
            // 쉼표와 공백 제거하고 추가
            let cleaned = word.trim().trim_end_matches(',').trim();
            if !cleaned.is_empty() {
                columns.push(cleaned.to_string());
            }
        }
    }

    columns
}

pub fn extract_tables(commands: &Vec<&str>) -> Vec<String> {
    // FROM 다음부터 (WHERE, ORDER BY, LIMIT 등 전까지)
    let mut tables = Vec::new();
    let mut in_from = false;

    for &word in commands {
        let upper = word;

        if upper == "from" {
            in_from = true;
            continue;
        }

        // FROM 이후 다른 키워드 만나면 중단
        if upper == "where"
            || upper == "order"
            || upper == "group"
            || upper == "limit"
            || upper == "join"
        {
            break;
        }

        if in_from {
            // 쉼표와 공백 제거하고 추가
            let cleaned = word.trim().trim_end_matches(',').trim();
            if !cleaned.is_empty() {
                tables.push(cleaned.to_string());
            }
        }
    }

    tables
}


pub fn extract_where_conditions(commands: &Vec<&str>) -> Option<HashMap<String, String>> {
    let mut conditions = HashMap::new();
    let mut in_where = false;
    let mut tokens = Vec::new();

    // WHERE부터 다른 키워드 전까지 토큰 수집
    for &word in commands {
        let lower = word.to_lowercase();

        if lower == "where" {
            in_where = true;
            continue;
        }

        // WHERE 이후 다른 키워드 만나면 중단
        if lower == "order" || lower == "group" || lower == "limit" || lower == ";" {
            break;
        }

        if in_where {
            tokens.push(word);
        }
    }

    if tokens.is_empty() {
        return None;
    }

    // 토큰을 순회하면서 "column = value" 패턴 찾기
    let mut i = 0;
    while i < tokens.len() {
        // AND/OR 스킵
        if tokens[i].to_lowercase() == "and" || tokens[i].to_lowercase() == "or" {
            i += 1;
            continue;
        }

        // "column = value" 패턴 파싱
        if i + 2 < tokens.len() && tokens[i + 1] == "=" {
            let column = tokens[i].trim().to_string();
            let value = tokens[i + 2]
                .trim()
                .trim_matches('\'')  // 작은따옴표 제거
                .trim_matches('"')   // 큰따옴표 제거
                .to_string();
            
            conditions.insert(column, value);
            i += 3; // column, =, value 건너뛰기
        } else {
            i += 1;
        }
    }

    if conditions.is_empty() {
        None
    } else {
        Some(conditions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_where_simple() {
        let cmd = vec!["select", "name", "from", "apples", "where", "color", "=", "'Yellow'"];
        let conditions = extract_where_conditions(&cmd);
        
        let mut expected = HashMap::new();
        expected.insert("color".to_string(), "Yellow".to_string());
        
        assert_eq!(conditions, Some(expected));
    }

    #[test]
    fn test_extract_where_no_quotes() {
        let cmd = vec!["select", "name", "from", "apples", "where", "id", "=", "5"];
        let conditions = extract_where_conditions(&cmd);
        
        let mut expected = HashMap::new();
        expected.insert("id".to_string(), "5".to_string());
        
        assert_eq!(conditions, Some(expected));
    }

    #[test]
    fn test_extract_where_multiple() {
        let cmd = vec![
            "select", "name", "from", "apples",
            "where", "color", "=", "'Yellow'",
            "and", "id", "=", "3"
        ];
        let conditions = extract_where_conditions(&cmd);
        
        let mut expected = HashMap::new();
        expected.insert("color".to_string(), "Yellow".to_string());
        expected.insert("id".to_string(), "3".to_string());
        
        assert_eq!(conditions, Some(expected));
    }

    #[test]
    fn test_extract_where_none() {
        let cmd = vec!["select", "name", "from", "apples"];
        let conditions = extract_where_conditions(&cmd);
        assert_eq!(conditions, None);
    }

    #[test]
    fn test_extract_where_double_quotes() {
        let cmd = vec!["select", "name", "from", "apples", "where", "color", "=", "\"Yellow\""];
        let conditions = extract_where_conditions(&cmd);
        
        let mut expected = HashMap::new();
        expected.insert("color".to_string(), "Yellow".to_string());
        
        assert_eq!(conditions, Some(expected));
    }

    #[test]
    fn test_extract_where_with_order() {
        let cmd = vec![
            "select", "*", "from", "apples", 
            "where", "color", "=", "'Red'",
            "order", "by", "name"
        ];
        let conditions = extract_where_conditions(&cmd);
        
        let mut expected = HashMap::new();
        expected.insert("color".to_string(), "Red".to_string());
        
        assert_eq!(conditions, Some(expected));
    }
}