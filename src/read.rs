use std::fs::File;
use std::io::{prelude::*};
use std::{error::Error, io::{Seek, SeekFrom}};


pub struct SqliteRead{}

impl SqliteRead{
    pub fn page_size(file_handler: &mut File) -> Result<u16, Box<dyn Error>>{
        file_handler.seek(SeekFrom::Start(0))?;
        let mut header = [0; 18];
        file_handler.read(&mut header)?;
        let page_size = u16::from_be_bytes([header[16], header[17]]) as u16;
        Ok(page_size)
    }



    pub fn table_count(file_handler: &mut File) -> Result<u16, Box<dyn Error>>{
        file_handler.seek(SeekFrom::Start(0))?;
        let mut header = [0; 105];
        file_handler.read(&mut header)?;
        let table_count = u16::from_be_bytes([header[103], header[104]]) as u16;
        Ok(table_count)
    }

    
    /// doesn't works for first page. choose another target page.  
    pub fn row_count(page: & Vec<u8>) -> Result<u16, Box<dyn Error>>{
        let table_count = u16::from_be_bytes([page[3], page[4]]) as u16;
        Ok(table_count)
    }


    /// not zero bound. start from 1.
    pub fn read_page_n(file_handler: &mut File, n: u16, page_size:u16) -> Result< Vec<u8> , Box<dyn Error>> {
        
        let mut single_page = vec![0;page_size as usize ];
        
        if n==1 {
            file_handler.seek(SeekFrom::Start(100))?;
        } else {
            file_handler.seek(SeekFrom::Start((page_size* (n-1) ) as u64))?;
        }
        
        file_handler.read(&mut single_page)?;

        Ok(single_page)
    }


    pub fn read_first_page(file_handler: &mut File, page_size:u16) -> Result< Vec<u8> , Box<dyn Error>> {
        Self::read_page_n(file_handler, 1, page_size-100)
    }




}