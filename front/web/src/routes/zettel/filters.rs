use pulldown_cmark::{BrokenLink, CowStr, Event, Options, Parser, Tag};
use std::fmt::Write;

pub fn render(s: &str) -> ::askama::Result<String> {
    let mut binding = accept_all_links;

    let mut parser = Parser::new_with_broken_link_callback(s, Options::all(), Some(&mut binding));
    let mut output = String::new();
    let mut writer = Writer { out: &mut output };
    while let Some(event) = parser.next() {
        if let Err(e) = writer.event(&mut parser, event) {
            return Err(askama::Error::Fmt(e));
        }
    }

    Ok(output)
}

struct Writer<'a> {
    out: &'a mut String,
}
impl Writer<'_> {
    pub(crate) fn event(&mut self, parser: &mut Parser, event: Event) -> std::fmt::Result {
        match event {
            Event::Start(tag) => self.start_tag(parser, tag),
            Event::End(tag) => self.end_tag(parser, tag),
            Event::Text(text) => self.text(parser, text),
            Event::Code(code) => self.code(parser, code),
            Event::Html(html) => self.html(parser, html),
            Event::FootnoteReference(reference) => self.footnote_reference(parser, reference),
            Event::SoftBreak => self.soft_break(),
            Event::HardBreak => self.hard_break(),
            Event::Rule => self.rule(),
            Event::TaskListMarker(marker) => self.task_list_marker(parser, marker),
        }
    }

    fn start_tag(&mut self, _parser: &mut Parser, tag: Tag) -> std::fmt::Result {
        match tag {
            Tag::Paragraph => write!(self.out, "<p>"),
            Tag::Heading(level, id, classes) => {
                write!(self.out, "<{level}")?;
                if let Some(id) = id {
                    write!(self.out, " id={id:?}")?;
                }
                if !classes.is_empty() {
                    write!(self.out, " class=\"")?;
                    for (idx, class) in classes.into_iter().enumerate() {
                        if idx != 0 {
                            self.out.push(' ');
                        }
                        *self.out += class;
                    }
                    self.out.push('"');
                }
                self.out.push('>');
                Ok(())
            }
            Tag::Strong => write!(self.out, "<b>"),
            Tag::Link(_, url, text) => write!(self.out, "<a href=\"{url}\" data-text=\"{text}\">["),
            tag => todo!("{tag:?}"),
        }
    }

    fn end_tag(&mut self, _parser: &mut Parser, tag: Tag) -> std::fmt::Result {
        match tag {
            Tag::Paragraph => write!(self.out, "</p>"),
            Tag::Heading(level, ..) => write!(self.out, "</{level}>"),
            Tag::Strong => write!(self.out, "</b>"),
            Tag::Link(_, _, _) => write!(self.out, "]</a>"),
            tag => todo!("{tag:?}"),
        }
    }

    fn text(&mut self, _parser: &mut Parser, text: CowStr) -> std::fmt::Result {
        *self.out += text.as_ref();
        Ok(())
    }

    fn code(&mut self, _parser: &mut Parser, _code: CowStr) -> std::fmt::Result {
        todo!()
    }

    fn html(&mut self, _parser: &mut Parser, _html: CowStr) -> std::fmt::Result {
        todo!()
    }

    fn footnote_reference(&mut self, _parser: &mut Parser, _reference: CowStr) -> std::fmt::Result {
        todo!()
    }

    fn soft_break(&mut self) -> std::fmt::Result {
        todo!()
    }

    fn hard_break(&mut self) -> std::fmt::Result {
        todo!()
    }

    fn rule(&mut self) -> std::fmt::Result {
        todo!()
    }

    fn task_list_marker(&mut self, _parser: &mut Parser, _marker: bool) -> std::fmt::Result {
        todo!()
    }
}

fn accept_all_links(link: BrokenLink<'_>) -> Option<(CowStr<'_>, CowStr<'_>)> {
    Some((link.reference.clone(), link.reference))
}
