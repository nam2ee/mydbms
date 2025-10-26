use anyhow::{Result};
use std::fs::File;
use std::error::Error;



mod page;
mod util;
mod read;
mod sql_engine;

use page::Cell;

use crate::read::SqliteRead;
use sql_engine::sql_engine;

fn main() -> Result<(), Box<dyn Error >>{
    // Parse arguments
    let args = std::env::args().collect::<Vec<_>>();
    match args.len() {
        0 | 1 => panic!("Missing <database path> and <command>"),
        2 => panic!("Missing <command>"),
        _ => {}
    }

    // Parse command and act accordingly
    let command = &args[2];
    match command.as_str() {
        ".dbinfo" => {
            let mut file = File::open(&args[1])?;
            // The page size is stored at the 16th byte offset, using 2 bytes in big-endian order
            #[allow(unused_variables)]
            if let Ok(page_size) = SqliteRead::page_size(&mut file) { println!("database page size: {}", page_size);}
            if let Ok(table_count) = SqliteRead::table_count(&mut file) {println!("number of tables: {}", table_count);}; 
            
        }
        ".tables" => {
            let mut file = File::open(&args[1])?;
           
            let page_size = SqliteRead::page_size(&mut file)?;
            let table_count = SqliteRead::table_count(&mut file)?;
            let page_0 = SqliteRead::read_first_page(&mut file, page_size)?;


            let ptrs = Cell::read_cell_pointer_array(&page_0, table_count);
            let mut result:String = String::from("");

            for v in ptrs{
                let tmp = Cell::parse_cell(&page_0, (v-100) as usize );
                if let Ok(name) = tmp{
                    if name[1] != String::from("sqlite_sequence") {
                        result = result +  &format!("{}", name[1]);
                    }
                }
            }
            println!("{}", result);

        }
        sql => {
            let mut file = File::open(&args[1])?;
           
            let page_size = SqliteRead::page_size(&mut file)?;
            let table_count = SqliteRead::table_count(&mut file)?;
            let page_0 = SqliteRead::read_first_page(&mut file, page_size)?;
            let ptrs = Cell::read_cell_pointer_array(&page_0, table_count);

            let mut result =vec![];
            for v in ptrs{
                let metadata = Cell::parse_cell(&page_0, (v-100) as usize)?; //considering header...
                if metadata[1] != String::from("sqlite_sequence") {
                        result.push(vec![metadata[1].clone(),metadata[3].clone(), metadata[4].clone()]); // table name, table's page index 
                }
            }

            sql_engine(&mut file, sql, result, page_size);

        }
        
    }

    Ok(())
}


