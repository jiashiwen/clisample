use rustyline::completion::{Completer, Pair, escape, Candidate};
use rustyline::Context;
use rustyline::Result;
use rustyline::completion::Quote;
use std::borrow::Cow::{self, Borrowed, Owned};
use std::fs;
use std::path::{self, Path};
use memchr::memchr;

const DOUBLE_QUOTES_ESCAPE_CHAR: Option<char> = Some('\\');

cfg_if::cfg_if! {
    if #[cfg(unix)] {
        // rl_basic_word_break_characters, rl_completer_word_break_characters
        const DEFAULT_BREAK_CHARS: [u8; 18] = [
            b' ', b'\t', b'\n', b'"', b'\\', b'\'', b'`', b'@', b'$', b'>', b'<', b'=', b';', b'|', b'&',
            b'{', b'(', b'\0',
        ];
        const ESCAPE_CHAR: Option<char> = Some('\\');
        // In double quotes, not all break_chars need to be escaped
        // https://www.gnu.org/software/bash/manual/html_node/Double-Quotes.html
        const DOUBLE_QUOTES_SPECIAL_CHARS: [u8; 4] = [b'"', b'$', b'\\', b'`'];
    } else if #[cfg(windows)] {
        // Remove \ to make file completion works on windows
        const DEFAULT_BREAK_CHARS: [u8; 17] = [
            b' ', b'\t', b'\n', b'"', b'\'', b'`', b'@', b'$', b'>', b'<', b'=', b';', b'|', b'&', b'{',
            b'(', b'\0',
        ];
        const ESCAPE_CHAR: Option<char> = None;
        const DOUBLE_QUOTES_SPECIAL_CHARS: [u8; 1] = [b'"']; // TODO Validate: only '"' ?
    } else if #[cfg(target_arch = "wasm32")] {
        const DEFAULT_BREAK_CHARS: [u8; 0] = [];
        const ESCAPE_CHAR: Option<char> = None;
        const DOUBLE_QUOTES_SPECIAL_CHARS: [u8; 0] = [];
    }
}

#[derive(PartialEq)]
enum ScanMode {
    DoubleQuote,
    Escape,
    EscapeInDoubleQuote,
    Normal,
    SingleQuote,
}

/// A `Completer` for file and folder names.
pub struct SubCmdCompleter {
    break_chars: &'static [u8],
    double_quotes_special_chars: &'static [u8],
}


impl SubCmdCompleter {
    /// Constructor
    pub fn new() -> Self {
        Self {
            break_chars: &DEFAULT_BREAK_CHARS,
            double_quotes_special_chars: &DOUBLE_QUOTES_SPECIAL_CHARS,
        }
    }

    /// Takes the currently edited `line` with the cursor `pos`ition and
    /// returns the start position and the completion candidates for the
    /// partial path to be completed.
    pub fn complete_path(&self, line: &str, pos: usize) -> Result<(usize, Vec<Pair>)> {
        let (start, path, esc_char, break_chars, quote) =
            if let Some((idx, quote)) = find_unclosed_quote(&line[..pos]) {
                let start = idx + 1;
                if quote == Quote::Double {
                    (
                        start,
                        unescape(&line[start..pos], DOUBLE_QUOTES_ESCAPE_CHAR),
                        DOUBLE_QUOTES_ESCAPE_CHAR,
                        &self.double_quotes_special_chars,
                        quote,
                    )
                } else {
                    (
                        start,
                        Borrowed(&line[start..pos]),
                        None,
                        &self.break_chars,
                        quote,
                    )
                }
            } else {
                let (start, path) = extract_word(line, pos, ESCAPE_CHAR, self.break_chars);
                let path = unescape(path, ESCAPE_CHAR);
                (start, path, ESCAPE_CHAR, &self.break_chars, Quote::None)
            };
        let mut matches = filename_complete(&path, esc_char, break_chars, quote);
        #[allow(clippy::unnecessary_sort_by)]
            matches.sort_by(|a, b| a.display().cmp(b.display()));
        Ok((start, matches))
    }
}

impl Default for SubCmdCompleter {
    fn default() -> Self {
        Self::new()
    }
}

impl Completer for SubCmdCompleter {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Result<(usize, Vec<Pair>)> {
        self.complete_path(line, pos)
    }
}


fn filename_complete(
    path: &str,
    esc_char: Option<char>,
    break_chars: &[u8],
    quote: Quote,
) -> Vec<Pair> {
    #[cfg(feature = "with-dirs")]
    use dirs_next::home_dir;
    use std::env::current_dir;

    let sep = path::MAIN_SEPARATOR;
    let (dir_name, file_name) = match path.rfind(sep) {
        Some(idx) => path.split_at(idx + sep.len_utf8()),
        None => ("", path),
    };

    let dir_path = Path::new(dir_name);
    let dir = if dir_path.starts_with("~") {
        // ~[/...]
        #[cfg(feature = "with-dirs")]
            {
                if let Some(home) = home_dir() {
                    match dir_path.strip_prefix("~") {
                        Ok(rel_path) => home.join(rel_path),
                        _ => home,
                    }
                } else {
                    dir_path.to_path_buf()
                }
            }
        #[cfg(not(feature = "with-dirs"))]
            {
                dir_path.to_path_buf()
            }
    } else if dir_path.is_relative() {
        // TODO ~user[/...] (https://crates.io/crates/users)
        if let Ok(cwd) = current_dir() {
            cwd.join(dir_path)
        } else {
            dir_path.to_path_buf()
        }
    } else {
        dir_path.to_path_buf()
    };

    let mut entries: Vec<Pair> = Vec::new();

    // if dir doesn't exist, then don't offer any completions
    if !dir.exists() {
        return entries;
    }

    // if any of the below IO operations have errors, just ignore them
    if let Ok(read_dir) = dir.read_dir() {
        let file_name = normalize(file_name);
        for entry in read_dir.flatten() {
            if let Some(s) = entry.file_name().to_str() {
                let ns = normalize(s);
                if ns.starts_with(file_name.as_ref()) {
                    if let Ok(metadata) = fs::metadata(entry.path()) {
                        let mut path = String::from(dir_name) + s;
                        if metadata.is_dir() {
                            path.push(sep);
                        }
                        entries.push(Pair {
                            display: String::from(s),
                            replacement: escape(path, esc_char, break_chars, quote),
                        });
                    } // else ignore PermissionDenied
                }
            }
        }
    }
    entries
}

/// Remove escape char
pub fn unescape(input: &str, esc_char: Option<char>) -> Cow<'_, str> {
    let esc_char = if let Some(c) = esc_char {
        c
    } else {
        return Borrowed(input);
    };
    if !input.chars().any(|c| c == esc_char) {
        return Borrowed(input);
    }
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars();
    while let Some(ch) = chars.next() {
        if ch == esc_char {
            if let Some(ch) = chars.next() {
                if cfg!(windows) && ch != '"' {
                    // TODO Validate: only '"' ?
                    result.push(esc_char);
                }
                result.push(ch);
            } else if cfg!(windows) {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }
    Owned(result)
}


#[cfg(any(windows, target_os = "macos"))]
fn normalize(s: &str) -> Cow<str> {
    // case insensitive
    Cow::Owned(s.to_lowercase())
}

pub fn extract_word<'l>(
    line: &'l str,
    pos: usize,
    esc_char: Option<char>,
    break_chars: &[u8],
) -> (usize, &'l str) {
    let line = &line[..pos];
    if line.is_empty() {
        return (0, line);
    }
    let mut start = None;
    for (i, c) in line.char_indices().rev() {
        if let (Some(esc_char), true) = (esc_char, start.is_some()) {
            if esc_char == c {
                // escaped break char
                start = None;
                continue;
            } else {
                break;
            }
        }
        if c.is_ascii() && memchr(c as u8, break_chars).is_some() {
            start = Some(i + c.len_utf8());
            if esc_char.is_none() {
                break;
            } // else maybe escaped...
        }
    }

    match start {
        Some(start) => (start, &line[start..]),
        None => (0, line),
    }
}

/// try to find an unclosed single/double quote in `s`.
/// Return `None` if no unclosed quote is found.
/// Return the unclosed quote position and if it is a double quote.
fn find_unclosed_quote(s: &str) -> Option<(usize, Quote)> {
    let char_indices = s.char_indices();
    let mut mode = ScanMode::Normal;
    let mut quote_index = 0;
    for (index, char) in char_indices {
        match mode {
            ScanMode::DoubleQuote => {
                if char == '"' {
                    mode = ScanMode::Normal;
                } else if char == '\\' {
                    // both windows and unix support escape in double quote
                    mode = ScanMode::EscapeInDoubleQuote;
                }
            }
            ScanMode::Escape => {
                mode = ScanMode::Normal;
            }
            ScanMode::EscapeInDoubleQuote => {
                mode = ScanMode::DoubleQuote;
            }
            ScanMode::Normal => {
                if char == '"' {
                    mode = ScanMode::DoubleQuote;
                    quote_index = index;
                } else if char == '\\' && cfg!(not(windows)) {
                    mode = ScanMode::Escape;
                } else if char == '\'' && cfg!(not(windows)) {
                    mode = ScanMode::SingleQuote;
                    quote_index = index;
                }
            }
            ScanMode::SingleQuote => {
                if char == '\'' {
                    mode = ScanMode::Normal;
                } // no escape in single quotes
            }
        };
    }
    if ScanMode::DoubleQuote == mode || ScanMode::EscapeInDoubleQuote == mode {
        return Some((quote_index, Quote::Double));
    } else if ScanMode::SingleQuote == mode {
        return Some((quote_index, Quote::Single));
    }
    None
}