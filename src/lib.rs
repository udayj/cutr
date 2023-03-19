use crate::Extract::*;
use clap::{App, Arg};
use std::{error::Error, 
    ops::Range, 
    num::NonZeroUsize,
    io::{self, BufRead, BufReader},
    fs::File,
};
use regex::Regex;
use csv::{ReaderBuilder, StringRecord};

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

    for filename in &config.files {

        match open(filename) {

            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(file) => {
                
                match &config.extract {

                    Chars(range) => {
                        
                            for line in file.lines() {

                                let actual_line = line.unwrap();
                                println!("{}",extract_chars(&actual_line, range));
    
                        }
                    
                    },
                    Bytes(range) => {

                        for line in file.lines() {
                            let actual_line = line.unwrap();
                            println!("{}", extract_bytes(&actual_line, range));
                        }
                    },
                    Fields(range) => {

                        let mut reader = ReaderBuilder::new()
                                                            .has_headers(false)
                                                            .delimiter(config.delimiter)
                                                            .from_reader(file);
                        let mut writer = csv::Writer::from_writer(io::stdout());
                        for record in reader.records() {
                                let mut result_str = String::new();
                                for pos in range {

                                    let records = record.as_ref().unwrap().iter().map(|v| format!("{}",v)).collect::<Vec<String>>();
                                    let actual_records = records.get(pos.start..pos.end).unwrap();
                                    
                                    result_str.push_str(actual_records.join(std::str::from_utf8(&[config.delimiter])?).as_str());
                                }
                                
                                println!("{}", result_str);
                                
                        }
                        //writer.flush();
                    }
                }
            }
        }
    }
    //println!("{:#?}", config);
    
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

    let delimiter = matches.value_of("delimiter").unwrap();
    let delim_bytes = delimiter.as_bytes();
    if delim_bytes.len() !=1 {
        return Err(From::from(format!(

            "--delim \"{}\" must be a single byte",
            delimiter
        )));
    }

    let fields = matches.value_of("fields").map(parse_pos).transpose()?;
    let bytes = matches.value_of("bytes").map(parse_pos).transpose()?;
    let chars = matches.value_of("chars").map(parse_pos).transpose()?;

    let extract = if let Some(field_pos) = fields {
        Fields(field_pos)
    } else if let Some(byte_pos) = bytes {
        Bytes(byte_pos)
    } else if let Some(char_pos) = chars {
        Chars(char_pos)
    } else {
        return Err(From::from("Must have --fields, --bytes or --chars"));
    };

    Ok(
        Config {
            files: matches.values_of_lossy("files").unwrap(),
            delimiter: *delim_bytes.first().unwrap(),
            extract,
        }
    )
                
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

    let range_re = Regex::new(r"^(\d+)-(\d+)$").unwrap();

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

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {

    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?)))
    
    }
}

fn extract_chars(line: &str, char_pos: &[Range<usize>]) -> String {

    let mut result = String::new();
    let line_as_char = line.chars().collect::<Vec<char>>();
    //let new_str = "";
    //println!("{}",&new_str[0..1]);
    for pos in char_pos {

        
        let extracted_val = line_as_char.get(pos.start..pos.end);

        match extracted_val {

            None => result.push_str(""),
            Some(val) => result.push_str(String::from_iter(
                val.iter()
            ).as_str())
        }
        
    }
    
    result
}

fn extract_bytes(line:&str, byte_pos: &[Range<usize>]) -> String {

    let mut result = String::new();
    let line_as_bytes = line.as_bytes();
    //let new_str = "";
    //println!("{}",&new_str[0..1]);
    for pos in byte_pos {

        
        let extracted_val = line_as_bytes.get(pos.start..pos.end);

        match extracted_val {

            None => result.push_str(""),
            Some(val) => result.push_str(String::from_utf8_lossy(val).into_owned().as_str()
                
            )
        }
        
    }
    
    result

}

#[cfg(test)]
mod unit_tests {
    use super::{extract_chars, extract_bytes, parse_pos};
    use csv::StringRecord;

    #[test]
    fn test_extract_chars() {
        assert_eq!(extract_chars("", &[0..1]), "".to_string());
        assert_eq!(extract_chars("ábc", &[0..1]), "á".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 2..3]), "ác".to_string());
        assert_eq!(extract_chars("ábc", &[0..3]), "ábc".to_string());
        assert_eq!(extract_chars("ábc", &[2..3, 1..2]), "cb".to_string());
        assert_eq!(
            extract_chars("ábc", &[0..1, 1..2, 4..5]),
            "áb".to_string()
        );
    }

    #[test]
    fn test_extract_bytes() {
        assert_eq!(extract_bytes("ábc", &[0..1]), "�".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2]), "á".to_string());
        assert_eq!(extract_bytes("ábc", &[0..3]), "áb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..4]), "ábc".to_string());
        assert_eq!(extract_bytes("ábc", &[3..4, 2..3]), "cb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2, 5..6]), "á".to_string());
    }
}