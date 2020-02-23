// https://riptutorial.com/rust/example/4404/a-simple-tcp-client-and-server-application--echo
// https://docs.rs/byteorder/1.1.0/byteorder/trait.ReadBytesExt.html
// https://doc.rust-lang.org/std/primitive.f32.html#method.from_le_bytes

// use std::net::{TcpStream};
use std::fs::File;
// use std::io::prelude::*;
use std::io::Read;
use std::io::{BufRead, BufReader};
// use std::str::from_utf8;
// use std::rc::Rc;
use std::borrow::Cow;

// let cube = r#"ply"#;

enum FormatType {
    ASCII,
    BinaryLittleEndian,
    BinaryBigEndian,
}

type FormatVersion = String;

type Format = (FormatType, FormatVersion);

type Comment = String;

enum SignedIntegerType {
    Char,
    Short,
    Int,
}

enum UnsignedIntegerType {
    UChar,
    UShort,
    UInt,
}

enum IntegerType {
    Signed(SignedIntegerType),
    Unsigned(UnsignedIntegerType),
}

enum FloatType {
    Float,
    Double,
}

enum ScalarType {
    Integer(IntegerType),
    Float(FloatType),
}

type ListType = (UnsignedIntegerType, ScalarType);

enum DataType {
    Scalar(ScalarType),
    List(ListType),
}

/*
enum PropertyType {
    X,
    Y,
    Z,
    Red,
    Green,
    Blue,
}
*/

struct Property {
    data_type: DataType,
    name: String,
}

/*
enum ElementType {
    Vertex,
    Edge,
    Face,
    Custom(String),
}
*/

struct Element {
    // element_type: ElementType,
    name: String,
    count: u64,
    properties: Vec<Property>,
}

struct Header {
    format: Format,
    comments: Vec<Comment>,
    elements: Vec<Element>,
}

fn parse_format_line(line: &str) -> Result<Format, String> {
    let mut iter = line.split_whitespace();
    let format_type = iter
        .next()
        .ok_or("format type not specified".to_string())
        .and_then(|format_type_str| match format_type_str {
            "ASCII" => Ok(FormatType::ASCII),
            "binary_little_endian" => Ok(FormatType::BinaryLittleEndian),
            "binary_big_endian" => Ok(FormatType::BinaryBigEndian),
            _ => Err(format!(
                "invalid format type specified: {}",
                format_type_str
            )),
        })?;

    let version = iter
        .next()
        .ok_or("format type not specified".to_string())
        .and_then(|version_str| match version_str {
            "1.0" => Ok(version_str),
            _ => Err(format!(
                "invalid or unsupported format version: {}",
                version_str
            )),
        })?;

    iter.next().map_or(Ok(()), |extra_str| {
        Err(format!("extra data in format line: {}", extra_str))
    })?;

    Ok((format_type, version.to_string()))
}

fn parse_comment_line(comment_line: &str) -> Result<Comment, String> {
    Ok(comment_line.to_string())
}

fn parse_element_line(line: &str) -> Result<Element, String> {
    let mut iter = line.split_whitespace();
    let name = iter
        .next()
        .ok_or("element name not specified".to_string())?;

    let count: u64 = iter
        .next()
        .ok_or("element count not specified".to_string())
        .and_then(|count_str| {
            count_str
                .parse::<u64>()
                .map_err(|_| "invalid element count (unsigned integer expected)".to_string())
        })?;

    iter.next().map_or(Ok(()), |extra_str| {
        Err(format!("extra data in element line: {}", extra_str))
    })?;

    Ok(Element {
        name: name.to_string(),
        count,
        properties: Vec::new(),
    })
}

/*
enum HeaderKey<'a> {
    PLY,
    Format,
    Comment(&'a str),
    Element,
    Property,
    EndHeader,
}
*/

enum HeaderKey {
    Start,
    Format,
    Comment,
    Element,
    Property,
    End,
}

type HeaderLine<'a> = (HeaderKey, &'a str);

struct HeaderReader<Stream: Read> {
    reader: BufReader<Stream>,
    line_string: String,
    line_count: u32,
}

impl<Stream: Read> HeaderReader<Stream> {
    pub fn new(stream: Stream) -> Self {
        Self {
            reader: BufReader::new(stream),
            line_string: String::with_capacity(256),
            line_count: 0,
        }
    }

    fn trim_line_endings<'a>(line: &'a String) -> &'a str {
        let line_ending_chars: &'static [_] = &['\n', '\r'];
        line.trim_end_matches(line_ending_chars)
    }
    // fn parse_header_line<'a>(line: &'a str) -> Result<HeaderKey, String> {
    fn parse_header_line<'a>(line: &'a str) -> Result<HeaderLine<'a>, String> {
        let (keyword, value) = match line.find(' ') {
            Some(i) => (&line[0..i], &line[i + 1..]),
            None => (line, ""),
        };

        match keyword {
            "ply" => Ok(HeaderKey::Start),
            "format" => Ok(HeaderKey::Format),
            "comment" => Ok(HeaderKey::Comment),
            "element" => Ok(HeaderKey::Element),
            "property" => Ok(HeaderKey::Property),
            "end_header" => Ok(HeaderKey::End),
            _ => Err(format!("invalid keyword in header: {}", keyword)),
        }
        .map(|key| (key, value))
    }

    pub fn read_line(&mut self) -> Result<HeaderLine, String> {
        self.line_string.clear();
        match self.reader.read_line(&mut self.line_string) {
            Ok(_) => {
                self.line_count = self.line_count + 1;
                Self::parse_header_line(Self::trim_line_endings(&self.line_string))
            }
            Err(e) => Err(e.to_string()),
        }
        .map_err(|e| format!("line {}: {}", self.line_count, e))
    }
}

/*
fn trim_line_endings<'a>(line: &'a String) -> &'a str {
    let line_ending_chars: &'static [_] = &['\n', '\r'];
    // let ss = "ACTG".repeat(10);
    // let sss: Cow<'a, str> = s.into();
    // Cow::Borrowed(
    // Cow::Owned(s.to_ascii_lowercase()
    line.trim_end_matches(line_ending_chars)
}

fn read_header_line<'a, Reader: BufRead>(
    reader: &mut Reader,
    line_string: &'a mut String,
    line_count: &mut u32,
) -> Result<HeaderKey<'a>, String> {
    line_string.clear();
    match reader.read_line(line_string) {
        Ok(_) => {
            *line_count = *line_count + 1;
            parse_header_line(trim_line_endings(line_string))
        }
        Err(e) => Err(e.to_string()),
    }
    .map_err(|e| format!("line {}: {}", line_count, e))
}
*/

// fn parse_header

fn ensure_empty(value: &str, error_message: &str) -> Result<(), String> {
    if value.trim_start().is_empty() {
        Ok(())
    } else {
        Err(error_message.to_string())
    }
}

fn parse_ply<Stream: Read>(stream: Stream) -> Result<(), String> {
    let mut header_reader = HeaderReader::new(stream);

    header_reader
        .read_line()
        .and_then(|(key, value)| match key {
            HeaderKey::Start => ensure_empty(value, "extra characters in start line"),
            _ => Err("start expected".to_string()),
        })?;

    let format = header_reader
        .read_line()
        .and_then(|(key, value)| match key {
            HeaderKey::Format => parse_format_line(value),
            _ => Err("format expected".to_string()),
        })?;

    let mut comments = Vec::<Comment>::new();
    let mut elements = Vec::<Element>::new();
    /*
    let n = (0..)
        .map(|_| header_reader.read_line()?)
        .map(|Ok((HeaderKey::Comment, value))| {
            parse_comment_line(value).and_then(|comment| Ok(comments.push(comment)))
        });
        */

    let n = loop {
        // let (key, value): HeaderLine = header_reader.read_line()?;
        match header_reader.read_line()? {
            (HeaderKey::Comment, value) => {
                parse_comment_line(value).and_then(|comment| Ok(comments.push(comment)))?
            }

            (key, value) => break (key, value),
        }
    };

    loop {
        // let (key, value): HeaderLine = header_reader.read_line()?;
        match header_reader.read_line()? {
            (HeaderKey::Comment, value) => {
                parse_comment_line(value).and_then(|comment| Ok(comments.push(comment)))?
            }

            (HeaderKey::Element, value) => {
                parse_element_line(value).and_then(|element| Ok(elements.push(element)))?
            }

            (HeaderKey::End, value) => {
                ensure_empty(value, "extra characters in end line")?;
                break;
            }

            _ => Err("comment, element or end expected".to_string())?,
        }
    }

    /*
    let read_header_line1 = || -> Result<HeaderKey, String> {
        line_string.clear();
        match reader.read_line(&mut line_string) {
            Ok(_) => {
                line_count = line_count + 1;
                parse_header_line(trim_line_endings(&line_string))
            }
            Err(e) => Err(e.to_string()),
        }
        .map_err(|e| format!("line {}: {}", line_count, e))
    };
    */

    /*
        let mut reader = BufReader::new(stream);

        let mut line_string = String::new();
        let mut line_count: u32 = 0;

        let ply = read_header_line(&mut reader, &mut line_string, &mut line_count)?;
        let format = read_header_line(&mut reader, &mut line_string, &mut line_count)?;
        {
            let comment: HeaderKey = read_header_line(&mut reader, &mut line_string, &mut line_count)?;
            match comment {
                HeaderKey::Comment(value) => println!("comment: {}", value),
                _ => (),
            };
        }
        {
            let comment: HeaderKey = read_header_line(&mut reader, &mut line_string, &mut line_count)?;
            match comment {
                HeaderKey::Comment(value) => println!("comment: {}", value),
                _ => (),
            };
        }
    */

    /*
    match comment1 {
        HeaderKey::Comment(value) => println!("comment1: {}", value),
    };
    */

    /*
        println!("parse_ply");
        let _hs = reader
            .lines()
            .map(|read_result| -> Result<HeaderKey, String> {
                println!("Line");
                match read_result {
                    Ok(line) => {
                        println!("Line: {}", line);
                        // let h =
                        //     parse_header_line(&line).map_err(|e| format!("line {}: {}", 1, e))?;
                        // Ok(&h)
                        Ok(HeaderKey::Element)
                    }
                    Err(e) => Err(format!("line {}: {}", 1, e)),
                }
            })
            .take_while(|item| item.is_ok());
    */
    // let _len = reader.read_line(&mut line)?;
    // let h = parse_header_line(&line)?;
    // println!("Data: {}", line);

    /*
    let mut data = [0 as u8; 6];
    reader.read_exact(&mut data).expect("Can't read data");
    match from_utf8(&data) {
      Ok(v) => println!("Data: {}", v),
      Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };
    */

    Ok(())
}

// fn main() -> std::io::Result<()> {
fn main() {
    // let stream = TcpStream::connect("google.com:80");
    let stream = File::open("./data/cube_ascii.ply");

    parse_ply(stream.expect("Error opening file"))
        .map_err(|e| format!("Error parsing file: {}", e))
        .unwrap();
}
