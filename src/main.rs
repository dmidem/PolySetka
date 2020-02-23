mod utils;

use utils::{ensure_empty, ensure_empty_iter, next_word_or_err, trim_line_endings};

use std::fs::File;
use std::io::Read;
use std::io::{BufRead, BufReader};

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

struct HeaderReader<Reader> {
  reader: Reader,
  line_string: String,
  line_count: u32,
}

impl<Reader: BufRead> HeaderReader<Reader> {
  pub fn new(reader: Reader) -> Self {
    Self {
      reader,
      line_string: String::with_capacity(256),
      line_count: 0,
    }
  }
  fn parse_header_line(line: &str) -> Result<HeaderLine, String> {
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

  pub fn read_line(&mut self) -> Result<HeaderLine, String> {
    self.line_string.clear();
    match self.reader.read_line(&mut self.line_string) {
      Ok(0) => Err("broken header".to_string()), // Ok(None),
      Ok(_) => {
        self.line_count += 1;
        Self::parse_header_line(&self.line_string)
      }
      Err(e) => Err(e.to_string()),
    }
    .map_err(|e| format!("line {}: {}", self.line_count, e))
  }
}

fn parse_header<Reader: BufRead>(reader: Reader) -> Result<(), String> {
  let mut header_reader = HeaderReader::new(reader);

  let mut format: Option<Format>;
  let mut comments = Vec::<Comment>::new();
  let mut elements = Vec::<Element>::new();

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

  Ok(())
}

fn parse_ply<Stream: Read>(stream: Stream) -> Result<(), String> {
  parse_header(BufReader::new(stream))
}

fn main() {
  // let stream = TcpStream::connect("google.com:80");
  let stream = File::open("./data/cube_ascii.ply");

  println!("ggg");

  parse_ply(stream.expect("Error opening file"))
    .map_err(|e| format!("Error parsing file: {}", e))
    .unwrap();
}
