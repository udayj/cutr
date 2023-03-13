use crate::Extract::*;
use clap::{App, Arg};
use std::{error::Error, ops::Range};

type MyResult<T> = Result<T, Box< dyn Error>>;
type PositionList = Vec<Range<usize>>;

#[derive(Debug)]
pub enum Extract {
    Fields(PositionList),
    Bytes(PositionList),
    Chars(PositionList),
}

#[derive(Debug)]
pub struct Config {

    files: Vec<String>,
    delimiter: u8,
    extract: Extract,
}

pub fn run(config: Config) -> MyResult<()> {

    println!("{:#?}", config);
    Ok(())
}

pub fn get_args() -> MyResult<Config> {

    let matches = App::new("cutr")
                    .version("0.1.0")
                    .author("udayj")
                    .about("Rust cut")
                    .arg(
                        Arg::with_name("files")
                            .value_name("FILES")
                            .help("Input File(s)")
                            .multiple(true)
                            .default_value("-")
                    )
                    .arg(
                        Arg::with_name("delimiter")
                            .value_name("DELIMITER")
                            .short("d")
                            .help("Delimiter")
                            .default_value(",")
                    )
                    .arg(
                        Arg::with_name("extract")
                            .short("f")
                            .value_name("EXTRACT FIELD")
                            .require_delimiter(true)
                            .multiple(true)
                    )
                    .get_matches();
}