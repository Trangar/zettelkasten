use crossterm::event::KeyCode;
use snafu::ResultExt;
use std::{
    collections::HashMap,
    io::{Read, Seek, SeekFrom, Write},
};
use tui::{
    style::{Color, Style},
    text::{Span, Spans},
};
use zettelkasten_shared::storage;

lazy_static::lazy_static! {
    /// https://regex101.com/r/wOR3xF/1
    /// matches:
    /// - [asd]
    /// - [asd](dsa)
    /// but not:
    /// - `[asd]
    /// - `[asd](dsa)
    static ref LINK_REGEX: regex::Regex = regex::Regex::new(r#"(?:^|[^`])(\[[^\]]+\])(\([^)]+\))?"#).unwrap();
}

pub struct RenderStyle {
    pub link_style: Style,
    pub link_highlight_style: Style,
}

impl Default for RenderStyle {
    fn default() -> Self {
        Self {
            link_style: Style::default().fg(Color::Yellow),
            link_highlight_style: Style::default().bg(Color::Yellow),
        }
    }
}

pub struct ParsedZettel<'a> {
    pub links: HashMap<String, &'a str>,
    pub link_char_size: usize,
    pub lines: Vec<Spans<'a>>,
}

impl<'a> ParsedZettel<'a> {
    #[allow(clippy::needless_pass_by_value, clippy::similar_names)]
    pub fn parse(
        zettel: &'a storage::Zettel,
        disallowed: &[char],
        render_links: bool,
        style: RenderStyle,
    ) -> ParsedZettel<'a> {
        let mut lines = Vec::new();
        let mut links = HashMap::new();

        let allowed_chars = ('a'..='z')
            .filter(|c| !disallowed.contains(c))
            .collect::<Vec<_>>();
        let matches = LINK_REGEX.find_iter(&zettel.body).count();

        let mut naming_scheme = if matches <= allowed_chars.len() {
            NamingScheme::single(allowed_chars)
        } else if matches <= allowed_chars.len().pow(2) {
            NamingScheme::double(allowed_chars)
        } else {
            panic!(
            "Too many links in document: {matches}, there are only enough characters for {} links",
            allowed_chars.len().pow(2)
        );
        };

        for mut line in zettel.body.lines() {
            let mut parts = Vec::new();

            for link in LINK_REGEX.captures_iter(line) {
                let text = link.get(1).unwrap();
                let maybe_link = link.get(2);
                let char = naming_scheme.next().expect("Ran out of characters");
                if text.start() > 0 {
                    parts.push(Span::raw(&line[..text.start()]));
                }
                if render_links {
                    // if we're rendering links, show the next CHAR as a highlight on top of the link
                    parts.push(Span::styled(
                        format!("[{char}]"),
                        style.link_highlight_style,
                    ));
                    // Render the rest of the link
                    let range = text.range();
                    let range = (range.start + 2 + char.len())..range.end;
                    parts.push(Span::styled(&line[range], style.link_style));
                } else {
                    parts.push(Span::styled(&line[text.range()], style.link_style));
                }
                let end = maybe_link.map_or_else(|| text.end(), |l| l.end());
                line = &line[end..];

                let url = if let Some(url) = maybe_link {
                    url.as_str().trim_start_matches('(').trim_end_matches(')')
                } else {
                    text.as_str().trim_start_matches('[').trim_end_matches(']')
                };
                links.insert(char, url);
            }
            if !line.is_empty() {
                parts.push(line.into());
            }
            lines.push(Spans(parts));
        }

        ParsedZettel {
            links,
            link_char_size: naming_scheme.link_char_size(),
            lines,
        }
    }
}

enum NamingScheme {
    Single {
        idx: usize,
        chars: Vec<char>,
    },
    Double {
        first_idx: usize,
        second_idx: usize,
        chars: Vec<char>,
    },
}

impl NamingScheme {
    fn single(chars: Vec<char>) -> Self {
        Self::Single { idx: 0, chars }
    }
    fn double(chars: Vec<char>) -> Self {
        Self::Double {
            first_idx: 0,
            second_idx: 0,
            chars,
        }
    }

    fn link_char_size(&self) -> usize {
        match self {
            Self::Single { .. } => 1,
            Self::Double { .. } => 2,
        }
    }
}

impl Iterator for NamingScheme {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Single { idx, chars } => {
                let result = chars.get(*idx)?.to_string();
                *idx += 1;
                Some(result)
            }
            Self::Double {
                first_idx,
                second_idx,
                chars,
            } => {
                let first = chars.get(*first_idx)?;
                let second = chars.get(*second_idx)?;
                *second_idx += 1;
                if *second_idx >= chars.len() {
                    *second_idx = 0;
                    *first_idx += 1;
                }
                Some(format!("{}{}", first, second))
            }
        }
    }
}

pub fn edit(zettel: &storage::Zettel, tui: &mut crate::Tui) -> super::Result<Option<String>> {
    let editor = if let Some(editor) = &tui.system_config.terminal_editor {
        editor
    } else {
        super::alert(tui.terminal, |cb| {
            cb.title("Could not edit zettel")
                .text("No terminal editor configured")
                .text("Please set one up in sys:config")
                .action(KeyCode::Enter, "Continue")
        })?;
        return Ok(None);
    };
    let mut tmp_file = tempfile::Builder::new()
        .suffix(".md")
        .tempfile()
        .context(super::IoSnafu)?;
    tmp_file
        .write_all(zettel.body.as_bytes())
        .context(super::IoSnafu)?;
    let _status = std::process::Command::new(editor)
        .arg(tmp_file.path())
        .status()
        .context(super::IoSnafu)?;

    tmp_file.seek(SeekFrom::Start(0)).context(super::IoSnafu)?;
    let mut result = String::new();
    tmp_file
        .read_to_string(&mut result)
        .context(super::IoSnafu)?;
    tui.terminal.clear().context(super::IoSnafu)?;
    Ok(Some(result))
}

pub(super) fn try_get_sys_page(
    tui: &mut crate::Tui,
    link: &str,
) -> super::Result<Option<super::ViewLayer>> {
    if let Some(page) = link.strip_prefix("sys:") {
        match page {
            "config" => Ok(Some(super::config::Config::new(tui).into())),
            _ => Err(super::Error::UnknownSysPage {
                page: page.to_string(),
            }),
        }
    } else {
        Ok(None)
    }
}
