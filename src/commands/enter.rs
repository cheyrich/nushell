use crate::commands::command::CommandAction;
use crate::commands::open::{fetch, parse_as_value};
use crate::errors::ShellError;
use crate::object::{Primitive, Value};
use crate::parser::registry::{CommandConfig, PositionalType};
use crate::prelude::*;
use std::path::PathBuf;

pub struct Enter;

impl Command for Enter {
    fn config(&self) -> CommandConfig {
        CommandConfig {
            name: self.name().to_string(),
            positional: vec![PositionalType::mandatory_block("path")],
            rest_positional: false,
            is_filter: false,
            is_sink: false,
            named: indexmap::IndexMap::new(),
        }
    }

    fn name(&self) -> &str {
        "enter"
    }

    fn run(&self, args: CommandArgs) -> Result<OutputStream, ShellError> {
        enter(args)
    }
}

pub fn enter(args: CommandArgs) -> Result<OutputStream, ShellError> {
    if args.len() == 0 {
        return Err(ShellError::maybe_labeled_error(
            "open requires a path or url",
            "missing path",
            args.name_span,
        ));
    }

    let span = args.name_span;

    let cwd = args
        .env()
        .lock()
        .unwrap()
        .front()
        .unwrap()
        .path()
        .to_path_buf();

    let full_path = PathBuf::from(cwd);

    let (file_extension, contents, contents_span) = match &args.expect_nth(0)?.item {
        Value::Primitive(Primitive::String(s)) => fetch(&full_path, s, args.expect_nth(0)?.span)?,
        _ => {
            return Err(ShellError::labeled_error(
                "Expected string value for filename",
                "expected filename",
                args.expect_nth(0)?.span,
            ));
        }
    };

    let mut stream = VecDeque::new();

    let file_extension = if args.has("raw") {
        None
    } else if args.has("json") {
        Some("json".to_string())
    } else if args.has("xml") {
        Some("xml".to_string())
    } else if args.has("ini") {
        Some("ini".to_string())
    } else if args.has("yaml") {
        Some("yaml".to_string())
    } else if args.has("toml") {
        Some("toml".to_string())
    } else {
        if let Some(ref named_args) = args.args.named {
            for named in named_args.iter() {
                return Err(ShellError::labeled_error(
                    "Unknown flag for enter",
                    "unknown flag",
                    named.1.span.clone(),
                ));
            }
            file_extension
        } else {
            file_extension
        }
    };

    match contents {
        Value::Primitive(Primitive::String(string)) => {
            stream.push_back(Ok(ReturnSuccess::Action(CommandAction::Enter(
                parse_as_value(file_extension, string, contents_span, span)?,
            ))));
        }

        other => stream.push_back(ReturnSuccess::value(other.spanned(contents_span))),
    };

    Ok(stream.into())
}