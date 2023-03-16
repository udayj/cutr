use crate::Extract::*;
use clap::{App, Arg};
use std::{error::Error, ops::Range, num::NonZeroUsize};
use regex::Regex;

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
                            .long("delim")
                            .help("Field Delimiter")
                            .default_value("\t")
                    )
                    .arg(
                        Arg::with_name("fields")
                            .short("f")
                            .long("fields")
                            .value_name("FIELDS")
                            .require_delimiter(true)
                            .multiple(true)
                            .conflicts_with_all(&["chars","bytes"])
                    )
                    .arg(
                        Arg::with_name("chars")
                            .short("c")
                            .long("chars")
                            .value_name("CHARS")
                            .require_delimiter(true)
                            .multiple(true)
                            .conflicts_with_all(&["fields","bytes"])
                    )
                    .arg(
                        Arg::with_name("bytes")
                            .short("b")
                            .long("bytes")
                            .value_name("BYTES")
                            .require_delimiter(true)
                            .multiple(true)
                            .conflicts_with_all(&["fields","chars"])
                    )
                    .get_matches();

                
}

fn parse_pos_basic(range: &str) -> MyResult<PositionList> {

    let mut result:PositionList = PositionList::new();

      for part in range.split(",") {

        if !part.contains("-") {
            
            result.push(Range{ start:part.parse::<usize>()?, end: part.parse::<usize>()?});
        }
        else {
            let parts:Vec<&str> = part.split("-").collect();
            result.push(Range{ start:parts[0].parse::<usize>()?, end: parts[1].parse::<usize>()?});
        }
        
    }
    return Ok(result);
}

fn parse_index(input: &str) -> Result<usize, String> {

    let value_error = || format!(
        "illegal list value: \"{}\"", input);

    input.starts_with('+')
        .then(|| Err(value_error()))
        .unwrap_or_else(
            || {
                input.parse::<NonZeroUsize>()
                .map(|n| usize::from(n) - 1)
                .map_err(|_| value_error())
            }
        )
    
}

fn parse_pos(range: &str) -> MyResult<PositionList> {

    let range_re = Regex::new(r"^(\d+)-(\d+)$)").unwrap();

    range.split(",")
        .into_iter()
        .map(|val| {
            parse_index(val).map(|n| n..n+1)
            .or_else(
                |e| {
                    range_re.captures(val).ok_or(e).and_then(|captures| {

                        let n1 = parse_index(&captures[1])?;
                        let n2 = parse_index(&captures[2])?;
                        
                        if n1>=n2 {
                            return Err(format!("First number in range ({}) \
                             must be lower than second number ({})", n1+1, n2+1));
                        }

                        Ok(n1..n2+1)
                    }
                )
                }
            )})
            .collect::<Result<_, _>>()
            .map_err(From::from)
}
