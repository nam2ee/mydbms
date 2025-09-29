use anyhow::{bail, Result};
use std::fs::File;
use std::io::prelude::*;

mod page;
mod util;

use page::Cell;

fn main() -> Result<()> {
    // Parse arguments
    let args = std::env::args().collect::<Vec<_>>();
    match args.len() {
        0 | 1 => bail!("Missing <database path> and <command>"),
        2 => bail!("Missing <command>"),
        _ => {}
    }

    // Parse command and act accordingly
    let command = &args[2];
    match command.as_str() {
        ".dbinfo" => {
            let mut file = File::open(&args[1])?;
            let mut header = [0; 4196];
            file.read_exact(&mut header)?;

            // The page size is stored at the 16th byte offset, using 2 bytes in big-endian order
            #[allow(unused_variables)]
            let page_size = u16::from_be_bytes([header[16], header[17]]);
            let tables = u16::from_be_bytes([header[103], header[104]]);

            println!("database page size: {}", page_size);
            println!("number of tables: {}", tables);
        }
        ".tables" => {
            let mut file = File::open(&args[1])?;
            let mut page = [0; 65636];
            file.read(&mut page)?;


            let table_count = u16::from_be_bytes([page[103], page[104]]);

            let ptrs = Cell::read_cell_pointer_array(&page, table_count);

            //println!("{:?}", ptrs);

            let mut result:String = String::from("");

            for v in ptrs{
                let k = Cell::parse_cell(&page, v as usize);

                if let Ok(name) = k{
                    if name[1] != String::from("sqlite_sequence") {
                        result = result +  &format!("{} ", name[1]);
                    }
                }
            }

            println!("{}", result);

        }
        _ => bail!("Missing or invalid command passed: {}", command),
    }

    Ok(())
}


