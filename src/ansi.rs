use std::convert::TryFrom;
use std::collections::{LinkedList, HashMap};
use regex::Regex;

#[derive(Clone, Debug)]
enum MarkupType {
    Color,
    Html
}

impl TryFrom<&str> for MarkupType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "c" | "C" => {
                Ok(Self::Color)
            },
            "p" | "P" => {
                Ok(Self::Html)
            },
            _ => Err(())
        }
    }
}

impl TryFrom<char> for MarkupType {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'c' | 'C' => {
                Ok(Self::Color)
            },
            'p' | 'P' => {
                Ok(Self::Html)
            },
            _ => Err(())
        }
    }
}

#[derive(Clone, Debug, Default)]
struct HtmlData {
    start: String,
    end: String,
}

enum AnsiGround {
    None,
    Fg,
    Bg
}

enum AnsiMode {
    None,
    Clear,
    Letters(String),
    Numbers(u8),
    Rgb(u8, u8, u8),
    Hex1(u16, u16, u16),
    Hex2(u16, u16, u16),
    Name(String)
}

impl Default for AnsiMode {
    fn default() -> Self {
        Self::None
    }
}


#[derive(Clone, Debug)]
struct Markup {
    pub order: usize,
    pub parent: Option<usize>,
    pub mode: MarkupType,
    pub start_text: String,
    pub end_text: String,
    pub html_start: String,
    pub html_end: String,
    pub ansi_bits: usize,
    pub ansi_off_bits: usize,
    pub ansi_fg_text: String,
    pub ansi_bg_text: String,
    pub ansi_fg_mode: AnsiMode,
    pub ansi_bg_mode: AnsiMode,
    pub ansi_reset: bool
}

impl Markup {
    pub fn new(order: usize, parent: Option<usize>, mode: MarkupType) -> Self {
        Self {
            order,
            parent,
            mode,
            start_text: "".to_string(),
            end_text: "".to_string(),
            html_start: "".to_string(),
            html_end: "".to_string(),
            ansi_bits: 0,
            ansi_off_bits: 0,
            ansi_fg_text: "".to_string(),
            ansi_bg_text: "".to_string(),
            ansi_fg_mode: Default::default(),
            ansi_bg_mode: Default::default(),
            ansi_reset: false
        }
    }
}

const TAG_START: char = '\x02';
const TAG_END: char = '\x03';

#[derive(Clone, Debug)]
pub struct AnsiString {
    clean: String,
    encoded: String,
    markups: Vec<Markup>,
    map: Vec<(Option<usize>, char)>
}

impl AnsiString {
    pub fn from_markup(src: impl Into<String>) -> Result<Self, String> {
        let mut markups = Vec::new();

        let mut map = Vec::with_capacity(src.len());
        let mut clean = "".to_string();

        let mut original = String::from(src);

        let mut mstack = Vec::new();
        let mut state: u8 = 0;
        let mut index: usize = 0;
        let mut depth: usize = 0;
        let mut tag: char = ' ';
        let mut counter = 0;

        for (i, c) in original.chars().enumerate() {
            match state {
                0 => {
                    if c == TAG_START {
                        state = 1;
                    } else {
                        clean.push(c);
                        if depth > 0 {
                            let val = (Some(index), c);
                            map.push(val);
                        } else {
                            map.push((None, c));
                        }
                    }
                },
                1 => {
                    tag = c;
                    state = 2;
                },
                2 => {
                    if c == '/' {
                        state = 4;
                    } else {
                        state = 3;

                        let parent = if depth > 0 {
                            Some(index)
                        } else {
                            None
                        };
                        if markups.len() > 0 {
                            counter += 1;
                        }
                        let mode = MarkupType::try_from(tag).unwrap();
                        let mut mark = Markup::new(counter, parent, mode);
                        mark.start_text.push(c);
                        markups.push(mark);
                        depth += 1;
                        index = counter;
                        mstack.push(counter);
                    }
                },
                3 => {
                    if c == TAG_END {
                        state = 0;
                    } else {
                        if let Some(m) = markups.get_mut(index) {
                            m.start_text.push(c);
                        }
                    }
                },
                4 => {
                    if c == TAG_END {
                        state = 0;
                        if let Some(op) = mstack.pop() {
                            if let Some(m) = markups.get(op) {
                                if let Some(ind) = m.parent {
                                    index = ind;
                                }
                            }
                        }
                        depth -= 1;
                    } else {
                        if let Some(m) = markups.get_mut(index) {
                            m.end_text.push(c);
                        }
                    }
                },
                _ => {
                    // This can't actually happen... right?
                }
            }
        }
        // we made it this far? Time to see if this string is valid...

        for m in &markups {
            Self.setup_markup(m);
        }

        Err("".to_string())

    }

    pub fn from_codes(codes: impl Into<String>, text: impl Into<String>) -> Result<Self, String> {

    }

    fn setup_markup(&mut m: Markup) -> Result<(), String> {
        match m.mode {
            MarkupType::Color => {

                Err("Whoops!".to_string())
            },
            MarkupType::Html => {
                m.start_html = format!("<{}>", m.start_text);
                let (tag, _) = m.start_text.split(' ');
                m.html_end = format!("</{}>", tag);
                Ok(())
            }
        }
    }

    pub fn encode(&self) -> String {

    }

    pub fn render_telnet(&self, ansi: bool, xterm: bool, mxp: bool) -> String {

    }

    pub fn validate_color_codes(src: impl Into<String>) -> Result<Vec<(AnsiMode, AnsiGround)>, String> {
        lazy_static! {
            static ref MATCH_MAP: &[(&'static str, Regex)] = [
                ("letters", Regex::new(r"(?i)^(?P<data>[a-z ]+)\b").unwrap()),
                ("numbers", Regex::new(r"^(?P<data>\d+)\b").unwrap()),
                ("rgb", Regex::new(r"(?i)^<(?P<red>\d{1,3})\s+(?P<green>\d{1,3})\s+(?P<blue>\d{1,3})>(\b)?")),
                ("hex1", Regex::new(r"(?i)^#(?P<data>[0-9A-F]{6})\b").unwrap()),
                ("hex2", Regex::new(r"(?i)^<#(?P<data>[0-9A-F]{6})>(\b)?").unwrap()),
                ("name", Regex::new(r"(?i)^\+(?P<data>\w+)\b").unwrap())
                ]
        }
    }
}