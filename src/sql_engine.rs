use crate::{page::Cell, read::SqliteRead};
use std::fs::File;

pub fn sql_engine(file_handler: &mut File, v: &str, metadata: Vec<Vec<String>>, page_size: u16) {
    let tmp_buffer: Vec<_> = v.split(" ").collect();
    let command = tmp_buffer[0];

    let target_columns = extract_columns(&tmp_buffer);
    let target_from_table = extract_tables(&tmp_buffer);

    match command {
        "SELECT" => {
            let _ = select(
                file_handler,
                target_columns,
                target_from_table,
                page_size,
                metadata,
            );
        }
        _ => {}
    }
}

fn extract_columns(commands: &Vec<&str>) -> Vec<String> {
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

fn extract_tables(commands: &Vec<&str>) -> Vec<String> {
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

fn select(
    file_handler: &mut File,
    target_columns: Vec<String>,
    froms: Vec<String>,
    page_size: u16,
    metadata: Vec<Vec<String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    for meta in &metadata {
        let table_name = meta[0].clone();
        let page_index = meta[1].clone();
        let sql = meta[2].clone();

        if froms.contains(&table_name) {
            let columns = Cell::parse_create_table(&sql)?;

            let page = SqliteRead::read_page_n(
                file_handler,
                page_index.parse::<u16>().unwrap(),
                page_size,
            )?;
            let row_count = SqliteRead::row_count(&page)?;
            let cell_ptrs = Cell::read_cell_pointer_array(&page, row_count);

            if target_columns.len() == 1 && target_columns[0] == "COUNT(*)" {
                println!("{}", row_count);
                continue;
            }

            let mut cell_list = vec![];
            for cell_ptr in cell_ptrs {
                let cell = Cell::parse_cell_as_map(&page, cell_ptr as usize, &columns)?;
                cell_list.push(cell);
            }

            for cell in cell_list {
                let mut row_output = Vec::new();
                for target in &target_columns {
                    if let Some(value) = cell.get(target) {
                        row_output.push(value.clone());
                    }
                }
                // 한 행의 모든 컬럼을 | 또는 공백으로 구분해서 출력
                println!("{}", row_output.join("|"));
            }
        } else {
            continue;
        }
    }

    Ok(())
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