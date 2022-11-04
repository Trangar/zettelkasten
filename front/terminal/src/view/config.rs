use std::path::PathBuf;

use crossterm::event::{Event, KeyCode};
use snafu::ResultExt;
use tui::{
    layout::Rect,
    style::{Color, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph},
};
use zettelkasten_shared::storage;

pub struct Config {
    form: Form,
}

impl Config {
    pub(crate) fn new(tui: &mut crate::Tui) -> Self {
        let mut form = Form {
            active: 0,
            fields: Vec::new(),
        };
        form.fields.push(Field {
            label: "User mode",
            ty: FieldTy::MultipleOption {
                options: vec![
                    "single user with auto login",
                    "single user with manual login",
                    "multiple users",
                ],
                selected: match tui.system_config.user_mode {
                    storage::UserMode::SingleUserAutoLogin => 0,
                    storage::UserMode::SingleUserManualLogin => 1,
                    storage::UserMode::MultiUser => 2,
                },
            },
            validator: None,
        });
        form.fields.push(Field {
            label: "Terminal editor (path)",
            ty: FieldTy::Text {
                value: if let Some(path) = &tui.system_config.terminal_editor {
                    path.display().to_string()
                } else {
                    String::new()
                },
            },
            validator: Some(validate_editor_path),
        });
        Self { form }
    }

    fn save(&self, tui: &mut crate::Tui) -> bool {
        let terminal_editor = self
            .form
            .get_value_by_label("Terminal editor (path)")
            .to_string();
        let config = storage::SystemConfig {
            terminal_editor: if terminal_editor.is_empty() {
                None
            } else {
                Some(PathBuf::from(terminal_editor))
            },
            user_mode: match self.form.get_idx_by_label("User mode") {
                0 => storage::UserMode::SingleUserAutoLogin,
                1 => storage::UserMode::SingleUserManualLogin,
                2 => storage::UserMode::MultiUser,
                _ => unreachable!(),
            },
        };
        if let Err(e) = zettelkasten_shared::block_on(tui.storage.update_config(&config)) {
            let _ = super::alert(tui.terminal, |f| {
                f.title("Could not save config")
                    .text(e.to_string())
                    .action(KeyCode::Enter, "Continue")
            });
            return false;
        }
        *tui.system_config = config;
        true
    }

    pub(crate) fn render(&mut self, tui: &mut crate::Tui) -> super::Result<Option<Transition>> {
        loop {
            let mut lines = Vec::new();
            for (idx, field) in self.form.fields.iter().enumerate() {
                let mut spans = Vec::new();
                spans.push(Span::raw(if idx == self.form.active { "> " } else { "  " }));
                spans.push(Span::raw(field.label));
                spans.push(Span::raw(" = "));
                field.ty.render(&mut spans, idx == self.form.active);

                if let Some(msg) = field.validator.as_ref().and_then(|v| v(field.ty.value())) {
                    spans.push(Span::raw("  "));
                    spans.push(Span::styled(msg, Style::default().fg(Color::Red)));
                }
                lines.push(Spans(spans));
            }
            let p = Paragraph::new(Text { lines })
                .block(Block::default().borders(Borders::ALL).title("Config"));

            let action = Paragraph::new(Text { lines: vec![
                Spans(vec![
                    Span::raw("Up/Down: select field, Left/Right: change option, Enter: Save, Esc: quit without saving")
                ])
            ]});
            let size = tui.terminal.size().context(super::TerminalSizeSnafu)?;

            tui.terminal
                .draw(|f| {
                    f.render_widget(
                        p,
                        Rect {
                            height: size.height - 1,
                            ..size
                        },
                    );
                    f.render_widget(
                        action,
                        Rect {
                            x: 0,
                            y: size.height - 1,
                            width: size.width,
                            height: 1,
                        },
                    );
                })
                .context(super::RenderFrameSnafu)?;

            let event = crossterm::event::read().context(super::EventSnafu)?;
            if let Event::Key(key_event) = event {
                match key_event.code {
                    KeyCode::Up => {
                        if self.form.active > 0 {
                            self.form.active -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if self.form.active + 1 < self.form.fields.len() {
                            self.form.active += 1;
                        }
                    }
                    KeyCode::Left => {
                        self.form.fields[self.form.active].ty.left();
                    }
                    KeyCode::Right => {
                        self.form.fields[self.form.active].ty.right();
                    }
                    KeyCode::Char(c) => {
                        self.form.fields[self.form.active].ty.input(c);
                    }
                    KeyCode::Backspace => {
                        self.form.fields[self.form.active].ty.backspace();
                    }
                    KeyCode::Esc => return Ok(Some(Transition::Pop)),
                    KeyCode::Enter => {
                        if self.save(tui) {
                            return Ok(Some(Transition::Pop));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

pub enum Transition {
    Pop,
}

struct Form {
    active: usize,
    fields: Vec<Field>,
}

struct Field {
    label: &'static str,
    ty: FieldTy,
    validator: Option<fn(&str) -> Option<&'static str>>,
}

enum FieldTy {
    MultipleOption {
        options: Vec<&'static str>,
        selected: usize,
    },
    Text {
        value: String,
    },
}

impl FieldTy {
    fn render(&self, spans: &mut Vec<Span>, active: bool) {
        match self {
            Self::MultipleOption { options, selected } => {
                spans.push(Span::styled(
                    if active && *selected > 0 { "< " } else { "  " },
                    Style::default().fg(Color::Yellow),
                ));
                spans.push(Span::raw(options[*selected]));
                spans.push(Span::styled(
                    if active && selected + 1 < options.len() {
                        " >"
                    } else {
                        "  "
                    },
                    Style::default().fg(Color::Yellow),
                ));
            }
            Self::Text { value } => {
                spans.push(Span::raw(value.clone()));
                if active {
                    spans.push(Span::raw("_"));
                }
            }
        }
    }

    fn input(&mut self, key: char) {
        if let Self::Text { value } = self {
            value.push(key);
        }
    }
    fn backspace(&mut self) {
        if let Self::Text { value } = self {
            value.pop();
        }
    }
    fn left(&mut self) {
        if let Self::MultipleOption { selected, .. } = self {
            if *selected > 0 {
                *selected -= 1;
            }
        }
    }
    fn right(&mut self) {
        if let Self::MultipleOption { options, selected } = self {
            if *selected + 1 < options.len() {
                *selected += 1;
            }
        }
    }

    fn value(&self) -> &str {
        match self {
            FieldTy::MultipleOption { options, selected } => options[*selected],
            FieldTy::Text { value } => value,
        }
    }
}

fn validate_editor_path(path: &str) -> Option<&'static str> {
    if path.trim().is_empty() {
        None
    } else if let Ok(metadata) = std::fs::metadata(path) {
        if metadata.is_file() {
            None
        } else {
            Some("Not a file")
        }
    } else {
        Some("Executable not found")
    }
}

impl Form {
    fn get_value_by_label(&self, arg: &str) -> &str {
        for field in &self.fields {
            if field.label == arg {
                return field.ty.value();
            }
        }
        panic!("Form field with label {arg:?} not found")
    }
    fn get_idx_by_label(&self, arg: &str) -> usize {
        for field in &self.fields {
            if field.label == arg {
                if let FieldTy::MultipleOption { selected, .. } = &field.ty {
                    return *selected;
                }
                panic!("Field {arg:?} is not a multiple choice");
            }
        }
        panic!("Form field with label {arg:?} not found")
    }
}
