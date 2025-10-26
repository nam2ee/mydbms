use crate::{page::Cell, read::SqliteRead, command_parse::extract_columns, command_parse::extract_tables};
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
