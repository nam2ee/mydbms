pub fn extract_columns(commands: &Vec<&str>) -> Vec<String> {
    // SELECT 다음부터 FROM 전까지
    let mut columns = Vec::new();
    let mut in_select = false;

    for &word in commands {
        let upper = word.to_uppercase();

        if upper == "SELECT" {
            in_select = true;
            continue;
        }

        if upper == "FROM" || upper == "WHERE" || upper == "ORDER" {
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
        let upper = word.to_uppercase();

        if upper == "FROM" {
            in_from = true;
            continue;
        }

        // FROM 이후 다른 키워드 만나면 중단
        if upper == "WHERE"
            || upper == "ORDER"
            || upper == "GROUP"
            || upper == "LIMIT"
            || upper == "JOIN"
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

fn extract_where_conditions(commands: &Vec<&str>) -> Option<Vec<(String,String)>> {
    todo!(); 
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_columns() {
        let cmd = vec!["SELECT", "name,", "color", "FROM", "apples"];
        let cols = extract_columns(&cmd);
        assert_eq!(cols, vec!["name", "color"]);
    }

    #[test]
    fn test_extract_columns_single() {
        let cmd = vec!["SELECT", "name", "FROM", "apples"];
        let cols = extract_columns(&cmd);
        assert_eq!(cols, vec!["name"]);
    }

    #[test]
    fn test_extract_columns_count() {
        let cmd = vec!["SELECT", "COUNT(*)", "FROM", "apples"];
        let cols = extract_columns(&cmd);
        assert_eq!(cols, vec!["COUNT(*)"]);
    }

    #[test]
    fn test_extract_tables() {
        let cmd = vec!["SELECT", "name", "FROM", "apples"];
        let tables = extract_tables(&cmd);
        assert_eq!(tables, vec!["apples"]);
    }

    #[test]
    fn test_extract_tables_multiple() {
        let cmd = vec!["SELECT", "name", "FROM", "apples,", "oranges"];
        let tables = extract_tables(&cmd);
        assert_eq!(tables, vec!["apples", "oranges"]);
    }
}