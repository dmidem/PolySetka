// https://riptutorial.com/rust/example/4404/a-simple-tcp-client-and-server-application--echo
// https://docs.rs/byteorder/1.1.0/byteorder/trait.ReadBytesExt.html
// https://doc.rust-lang.org/std/primitive.f32.html#method.from_le_bytes

mod utils;

use utils::{ensure_empty, ensure_empty_iter, next_word_or_err, trim_line_endings};

// use std::net::{TcpStream};
use std::fs::File;
// use std::io::prelude::*;
use std::io::Read;
use std::io::{BufRead, BufReader};
// use std::str::from_utf8;
// use std::rc::Rc;
// use std::borrow::Cow;
// use std::str::SplitAsciiWhitespace;
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
  let mut iter = line.split_ascii_whitespace();
  let format_type =
    next_word_or_err(&mut iter, "format type not specified").and_then(|format_type_str| {
      match format_type_str {
        "ascii" => Ok(FormatType::ASCII),
        "binary_little_endian" => Ok(FormatType::BinaryLittleEndian),
        "binary_big_endian" => Ok(FormatType::BinaryBigEndian),
        _ => Err(format!(
          "invalid format type specified: {}",
          format_type_str
        )),
      }
    })?;
  let version =
    next_word_or_err(&mut iter, "format version not specified").and_then(|version_str| {
      match version_str {
        "1.0" => Ok(version_str),
        _ => Err(format!(
          "invalid or unsupported format version: {}",
          version_str
        )),
      }
    })?;
  ensure_empty_iter(&mut iter, "format")?;
  Ok((format_type, version.to_string()))
}

fn parse_comment_line(comment_line: &str) -> Result<Comment, String> {
  Ok(trim_line_endings(comment_line).to_string())
}

fn parse_element_line(line: &str) -> Result<Element, String> {
  let mut iter = line.split_ascii_whitespace();
  let name = next_word_or_err(&mut iter, "element name not specified")?;

  let count: u64 =
    next_word_or_err(&mut iter, "element count not specified").and_then(|count_str| {
      count_str
        .parse::<u64>()
        .map_err(|_| "invalid element count (unsigned integer expected)".to_string())
    })?;

  ensure_empty_iter(&mut iter, "element")?;
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
  None,
  Start,
  Format,
  Comment,
  Element,
  Property,
  End,
}

type HeaderLine<'a> = (HeaderKey, &'a str, &'a str);

// struct HeaderReader<Stream: Read> {
// reader: BufReader<Stream>,
struct HeaderReader<Reader> {
  reader: Reader,
  line_string: String,
  line_count: u32,
}

// impl<Stream: Read> HeaderReader<Stream> {
impl<Reader: BufRead> HeaderReader<Reader> {
  // pub fn new(stream: Stream) -> Self {
  pub fn new(reader: Reader) -> Self {
    Self {
      // reader: BufReader::new(stream),
      reader,
      line_string: String::with_capacity(256),
      line_count: 0,
    }
  }
  /*
  fn trim_line_endings<'a>(line: &'a String) -> &'a str {
      let line_ending_chars: &'static [_] = &['\n', '\r'];
      line.trim_end_matches(line_ending_chars)
  }
  */
  /*
  // FIXME: use standard strip_suffix when it'll be apporved
  fn strip_suffix(s: &str, suffix: char) -> Option<&str> {
      if s.ends_with(suffix) {
          Some(&s[0..s.len() - 1])
      } else {
          None
      }
  }

  // fn trim_line_endings<'a>(line: &'a String) -> &'a str {
  #[cfg(_NO_)]
  fn trim_line_ending(line: &str) -> &str {
      Self::strip_suffix(line, '\n').map_or(line, |s| Self::strip_suffix(s, '\r').unwrap_or(s))
  }
  */
  // fn parse_header_line<'a>(line: &'a str) -> Result<HeaderKey, String> {
  // fn parse_header_line<'a>(line: &'a str) -> Result<HeaderLine<'a>, String> {
  fn parse_header_line(line: &str) -> Result<HeaderLine, String> {
    // match line.find(' ') {
    let (keyword, value) = {
      let mut it = line.splitn(2, |c: char| -> bool { c.is_ascii_whitespace() });
      // Rust guarantees left-to-right evaluation order
      (it.next().unwrap(), it.next().unwrap_or(""))
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
    .map(|key| (key, keyword, value))
  }

  // Result<Option<HeaderLine>, String> {
  pub fn read_line(&mut self) -> Result<HeaderLine, String> {
    self.line_string.clear();
    match self.reader.read_line(&mut self.line_string) {
      Ok(0) => Err("broken header".to_string()), // Ok(None),
      Ok(_) => {
        self.line_count += 1;
        Self::parse_header_line(&self.line_string)
        // Self::parse_header_line(Self::trim_line_ending(&self.line_string))
        // Some(Self::parse_header_line(&self.line_string)).transpose()
        // Self::parse_header_line(&self.line_string)
      }
      Err(e) => Err(e.to_string()),
    }
    .map_err(|e| format!("line {}: {}", self.line_count, e))
  }
}

/*
pub struct Lines<'a, B> {
    buf: B,
    str: String,
    c: &'a str,
}

impl<'a, B: BufRead> Iterator for Lines<'a, B> {
    type Item = Result<&'a str, String>;

    fn next(&mut self) -> Option<Result<&'a str, String>> {
        // let mut buf = String::new();
        match self.buf.read_line(&mut self.str) {
            Ok(0) => None,
            Ok(_n) => {
                if self.str.ends_with("\n") {
                    self.str.pop();
                    if self.str.ends_with("\r") {
                        self.str.pop();
                    }
                }
                self.c = self.str.as_ref();
                Some(Ok(self.c))
            }
            Err(e) => Some(Err(e.to_string())),
        }
    }
}
*/

/*
impl<'a, Stream: Read> Iterator for HeaderReader<'a, Stream> {
    type Item = Result<HeaderLine<'a>, String>;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.read_line())
    }
}
*/

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

/*
fn ensure_header_key<RequiredKey>(header_key: HeaderKey) -> Result<(), String> {
    match header_key {
        RequiredKey => Ok(()),
        _ => Err("unexpected header key".to_string()), // FIXME
    }
}
*/

/*
fn ensure_empty(value: &str, error_message: &str) -> Result<(), String> {
    if value.trim_start().is_empty() {
        Ok(())
    } else {
        Err(error_message.to_string())
    }
}
*/

// fn parse_header<Stream: Read>(stream: Stream) -> Result<(), String> {
//    let mut header_reader = HeaderReader::new(stream);
fn parse_header<Stream: Read>(stream: Stream) -> Result<(), String> {
  let mut header_reader = HeaderReader::new(stream);

  let mut format: Option<Format>;
  let mut comments = Vec::<Comment>::new();
  let mut elements = Vec::<Element>::new();
  /*
  header_reader
      .read_line()
      .and_then(|(key, value)| match key {
          HeaderKey::Start => ensure_empty(value, "extra characters after \"ply\" keyword"),
          _ => Err("\"ply\" keyword expected".to_string()),
      })?;

  let format = header_reader
      .read_line()
      .and_then(|(key, value)| match key {
          HeaderKey::Format => parse_format_line(value),
          _ => Err("\"format\" keyword expected".to_string()),
      })?;

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
          (HeaderKey::Comment, value) => comments.push(parse_comment_line(value)?),

          (key, value) => break (key, value),
      }
  };
  */

  let mut prev_header_key = HeaderKey::None;

  loop {
    let (key, keyword, value): HeaderLine = header_reader.read_line()?;
    println!("keyword: {}", keyword);
    match key {
      // https://stackoverflow.com/questions/31123882/how-to-map-a-parametrized-enum-from-a-generic-type-to-another
      HeaderKey::Start => {
        match prev_header_key {
          HeaderKey::None => Ok(()),
          _ => Err("\"ply\" keyword expected".to_string()),
        }?;
        ensure_empty(value, "ply")?
      }

      HeaderKey::Format => {
        match prev_header_key {
          HeaderKey::Start => Ok(()),
          _ => Err("\"format\" keyword expected".to_string()),
        }?;
        format = Some(parse_format_line(value)?)
      }

      HeaderKey::Comment => comments.push(parse_comment_line(value)?),
      HeaderKey::Element => elements.push(parse_element_line(value)?),
      // HeaderKey::Property => parse_property_line(value)?,
      HeaderKey::End => {
        ensure_empty(value, "end_header")?;
        break;
      }
      _ => return Err("\"comment\", \"element\" or \"end_header\" keywords expected".to_string()),
    }
    prev_header_key = key
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

fn parse_ply<Stream: Read>(stream: Stream) -> Result<(), String> {
  let mut header_reader = HeaderReader::new(stream);
  parse_header()
}
// fn main() -> std::io::Result<()> {
fn main() {
  // let stream = TcpStream::connect("google.com:80");
  let stream = File::open("./data/cube_ascii.ply");

  parse_ply(stream.expect("Error opening file"))
    .map_err(|e| format!("Error parsing file: {}", e))
    .unwrap();
}
