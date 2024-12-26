use serde::ser::{Serialize, SerializeStruct, Serializer};
use uiua::{
    format::{format_str, FormatConfig, FormatOutput},
    CodeSpan, InputSrc, Loc, Span,
};
use wasm_bindgen::{prelude::wasm_bindgen, JsError, JsValue};

#[derive(serde::Serialize, serde::Deserialize, Default)]
#[wasm_bindgen]
pub struct FormatConfigStruct {
    trailing_newline: Option<bool>,
    comment_space_after_hash: Option<bool>,
    multiline_indent: Option<i32>,
    align_comments: Option<bool>,
    indent_item_imports: Option<bool>,
}

#[wasm_bindgen]
impl FormatConfigStruct {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_trailing_newline(mut self, trailing_newline: bool) -> Self {
        self.trailing_newline = Some(trailing_newline);
        self
    }

    pub fn with_comment_space_after_hash(mut self, comment_space_after_hash: bool) -> Self {
        self.comment_space_after_hash = Some(comment_space_after_hash);
        self
    }

    pub fn with_multiline_indent(mut self, multiline_indent: i32) -> Self {
        self.multiline_indent = Some(multiline_indent);
        self
    }

    pub fn with_align_comments(mut self, align_comments: bool) -> Self {
        self.align_comments = Some(align_comments);
        self
    }

    pub fn with_indent_item_imports(mut self, indent_item_imports: bool) -> Self {
        self.indent_item_imports = Some(indent_item_imports);
        self
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct DocumentLocation {
    pub line: u16,
    pub column: u16,
}

impl DocumentLocation {
    pub fn add_line(&self, line: u16) -> Self {
        DocumentLocation {
            line: self.line + line,
            column: self.column,
        }
    }

    pub fn decrement_column(&self) -> Self {
        DocumentLocation {
            line: self.line,
            column: self.column - 1,
        }
    }
}

#[derive(serde::Serialize, Clone)]
pub struct DocumentSpan {
    pub src: UiuaInputSource,
    pub from: DocumentLocation,
    pub to: DocumentLocation,
}

impl DocumentSpan {
    fn fix_column(&self) -> Self {
        DocumentSpan {
            src: self.src.clone().into(),
            from: self.from.decrement_column(),
            to: self.to.decrement_column(),
        }
    }
}

impl From<CodeSpan> for DocumentSpan {
    fn from(span: CodeSpan) -> Self {
        DocumentSpan {
            src: span.src.into(),
            from: span.start.into(),
            to: span.end.into(),
        }
    }
}

impl From<Span> for DocumentSpan {
    fn from(span: Span) -> Self {
        match span {
            Span::Code(span) => DocumentSpan::from(span),
            Span::Builtin => DocumentSpan {
                src: UiuaInputSource::Builtin,
                from: DocumentLocation { line: 0, column: 0 },
                to: DocumentLocation { line: 0, column: 0 },
            },
        }
    }
}

impl From<Loc> for DocumentLocation {
    fn from(loc: Loc) -> Self {
        DocumentLocation {
            line: loc.line,
            column: loc.col,
        }
    }
}

#[derive(serde::Serialize)]
pub struct GlyphMapping {
    pub span_from: DocumentSpan,
    pub span_to: DocumentSpan,
}

impl From<(&CodeSpan, (Loc, Loc))> for GlyphMapping {
    fn from((span, (from, to)): (&CodeSpan, (Loc, Loc))) -> Self {
        GlyphMapping {
            span_from: DocumentSpan::from(span.clone()).fix_column(),
            span_to: DocumentSpan {
                src: span.src.clone().into(),
                from: DocumentLocation::from(from).add_line(1),
                to: DocumentLocation::from(to).add_line(1),
            },
        }
    }
}

#[derive(serde::Serialize)]
pub struct FormatOutputStruct {
    pub output: String,
    pub glyph_map: Vec<GlyphMapping>,
}

impl From<FormatOutput> for FormatOutputStruct {
    fn from(output: FormatOutput) -> Self {
        FormatOutputStruct {
            output: output.output,
            glyph_map: output
                .glyph_map
                .iter()
                .map(|(span, locs)| GlyphMapping::from((span, *locs)))
                .collect(),
        }
    }
}

#[derive(Clone)]
pub enum UiuaInputSource {
    String(usize),
    File(String),
    Macro(Box<DocumentSpan>),
    Builtin,
}

impl From<InputSrc> for UiuaInputSource {
    fn from(src: InputSrc) -> Self {
        match src {
            InputSrc::Str(id) => UiuaInputSource::String(id),
            InputSrc::File(path) => UiuaInputSource::File((*path).to_string_lossy().to_string()),
            InputSrc::Macro(src) => UiuaInputSource::Macro(Box::new(DocumentSpan::from(*src))),
        }
    }
}

impl Serialize for UiuaInputSource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            UiuaInputSource::String(id) => {
                let mut struct_ser = serializer.serialize_struct("UiuaInputSource", 2)?;
                struct_ser.serialize_field("type", &"string")?;
                struct_ser.serialize_field("id", id)?;
                struct_ser.end()
            }
            UiuaInputSource::File(path) => {
                let mut struct_ser = serializer.serialize_struct("UiuaInputSource", 2)?;
                struct_ser.serialize_field("type", &"file")?;
                struct_ser.serialize_field("path", path)?;
                struct_ser.end()
            }
            UiuaInputSource::Macro(span) => {
                let mut struct_ser = serializer.serialize_struct("UiuaInputSource", 2)?;
                struct_ser.serialize_field("type", &"macro")?;
                struct_ser.serialize_field("span", span)?;
                struct_ser.end()
            }
            UiuaInputSource::Builtin => {
                let mut struct_ser = serializer.serialize_struct("UiuaInputSource", 1)?;
                struct_ser.serialize_field("type", &"builtin")?;
                struct_ser.end()
            }
        }
    }
}

#[wasm_bindgen]
pub fn format_internal(code: String, config: FormatConfigStruct) -> Result<JsValue, JsError> {
    let mut format_config = FormatConfig::default();

    if let Some(trailing_newline) = config.trailing_newline {
        format_config.trailing_newline = trailing_newline;
    }

    if let Some(comment_space_after_hash) = config.comment_space_after_hash {
        format_config.comment_space_after_hash = comment_space_after_hash;
    }

    if let Some(multiline_indent) = config.multiline_indent {
        format_config.multiline_indent = multiline_indent as usize;
    }

    if let Some(align_comments) = config.align_comments {
        format_config.align_comments = align_comments;
    }

    if let Some(indent_item_imports) = config.indent_item_imports {
        format_config.indent_item_imports = indent_item_imports;
    }

    let format_output = format_str(&*code, &format_config)?;
    let output = FormatOutputStruct::from(format_output);
    Ok(serde_wasm_bindgen::to_value(&output)?)
}
