pub fn ensure_empty_iter<'a>(
  iter: &mut impl Iterator<Item = &'a str>,
  keyword: &str,
) -> Result<(), String> {
  iter.next().map_or(Ok(()), |extra_str| {
    Err(format!(
      "extra characters in \"{}\" line: {}",
      keyword, extra_str
    ))
  })
}

pub fn ensure_empty(value: &str, keyword: &str) -> Result<(), String> {
  ensure_empty_iter(&mut value.split_ascii_whitespace(), keyword)
}

pub fn trim_line_endings(line: &str) -> &str {
  let line_ending_chars: &'static [_] = &['\n', '\r'];
  line.trim_end_matches(line_ending_chars)
}

pub fn next_word_or_err<'a>(
  iter: &mut impl Iterator<Item = &'a str>,
  error_message: &str,
) -> Result<&'a str, String> {
  iter.next().ok_or_else(|| error_message.to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_ensure_empty_iter() {
    let mut iter = " abc 123  \n ".split_whitespace();
    assert!(ensure_empty_iter(&mut iter, "").is_err());
    assert!(ensure_empty_iter(&mut iter, "").is_err());
    assert!(ensure_empty_iter(&mut iter, "").is_ok());
  }

  #[test]
  fn test_ensure_empty() {
    assert!(ensure_empty("", "").is_ok());
    assert!(ensure_empty(" \t\r\n\n\r\t ", "").is_ok());
    assert!(ensure_empty("abc", "").is_err());
    assert!(ensure_empty("abc\n", "").is_err());
    assert!(ensure_empty("abc \n", "").is_err());
  }

  #[test]
  fn test_trim_line_endings() {
    assert_eq!(trim_line_endings(""), "");
    assert_eq!(trim_line_endings("\n"), "");
    assert_eq!(trim_line_endings("\r"), "");
    assert_eq!(trim_line_endings("\n\r"), "");
    assert_eq!(trim_line_endings("\r\n"), "");
    assert_eq!(trim_line_endings("\n\n\r\r"), "");
    assert_eq!(trim_line_endings("\r\r\n\n"), "");
    assert_eq!(trim_line_endings("\r\n\r\n"), "");
    assert_eq!(trim_line_endings(" \r\n"), " ");
    assert_eq!(trim_line_endings("\t\r\n"), "\t");
    assert_eq!(trim_line_endings("abc \t\t  \r\n"), "abc \t\t  ");
    assert_eq!(trim_line_endings("abc \t\t  "), "abc \t\t  ");
    assert_eq!(trim_line_endings("abc"), "abc");
  }

  #[test]
  fn test_next_word_or_err() {
    assert!(next_word_or_err(&mut "".split_whitespace(), "").is_err());
    assert_eq!(
      next_word_or_err(&mut "abc".split_whitespace(), ""),
      Ok("abc")
    );
  }
}
