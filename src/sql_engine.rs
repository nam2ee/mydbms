use crate::{page::Cell, read::SqliteRead, command_parse::extract_columns, command_parse::extract_tables, command_parse::extract_where_conditions};
use std::{collections::HashMap, fs::File};


pub fn sql_engine(file_handler: &mut File, v: &str, metadata: Vec<Vec<String>>, page_size: u16) {
    let tmp_buffer: Vec<_> = v.split(" ").collect();
    let command = tmp_buffer[0];

    let target_columns = extract_columns(&tmp_buffer);
    let target_from_table = extract_tables(&tmp_buffer);
    let where_conditions = extract_where_conditions(&tmp_buffer);

    match command {
        "select" => {
            let _ = select(
                file_handler,
                target_columns,
                target_from_table,
                where_conditions,
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
    where_conditions: Option<HashMap<String, String>>,
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

            if target_columns.len() == 1 && target_columns[0] == "count(*)" {
                println!("{}", row_count);
                continue;
            }

            let mut cell_list = vec![];
            for cell_ptr in cell_ptrs {
                let cell = Cell::parse_cell_as_map(&page, cell_ptr as usize, &columns)?;
                cell_list.push(cell);
            }

            cell_filtering(cell_list, &where_conditions, &target_columns);

            
        } else {
            continue;
        }
    }

    Ok(())
}


fn cell_filtering( cell_list:Vec<HashMap<String, String>> , condition_list: &Option<HashMap<String, String>>, target_columns: &Vec<String>) {
    let is_none = condition_list.is_none();


    if is_none {
        cell_list.iter().for_each( |cell| { 
            let mut row_output = Vec::new();
            for target in target_columns {

                    if let Some(value) = cell.get(target) {
                        row_output.push(value.clone());
                    }
                    
                }
                println!("{}", row_output.join("|"));
        });

    } else {
        
        cell_list.iter().for_each( |cell| { 
            let condition_map = condition_list.clone().unwrap();
            let mut row_output = Vec::new();
            let mut flag = true; 
            for target in target_columns {

                if let Some(value) = cell.get(target) {
                    row_output.push(value.clone());
                    let compe = condition_map.get(target);
                   
                    if compe.is_none() {
                        continue;
                    } else {
                        let compe = compe.unwrap();
                        if value.to_lowercase() != *compe {
                            flag = false;
                        }
                    }

                }
            }

            if flag {
                println!("{}", row_output.join("|"));
            }

        });

    }

}