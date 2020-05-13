//! Classes that store formatting options
use crate::avm1::{Avm1, Object, ScriptObject, TObject, Value};
use crate::context::UpdateContext;
use crate::tag_utils::SwfMovie;
use crate::xml::{Step, XMLDocument, XMLName, XMLNode};
use gc_arena::Collect;
use std::cmp::{min, Ordering};
use std::sync::Arc;

/// A set of text formatting options to be applied to some part, or the whole
/// of, a given text field.
///
/// Any property set to `None` is treated as undefined, which has different
/// meanings based on the context by which the `TextFormat` is used. For
/// example, when getting the format of a particular region of text, `None`
/// means that multiple regions of text apply. When setting the format of a
/// particular region of text, `None` means that the existing setting for that
/// property will be retained.
#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub struct TextFormat {
    pub font: Option<String>,
    pub size: Option<f64>,
    pub color: Option<swf::Color>,
    pub align: Option<swf::TextAlign>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub left_margin: Option<f64>,
    pub right_margin: Option<f64>,
    pub indent: Option<f64>,
    pub block_indent: Option<f64>,
    pub kerning: Option<bool>,
    pub leading: Option<f64>,
    pub letter_spacing: Option<f64>,
    pub tab_stops: Option<Vec<f64>>,
    pub bullet: Option<bool>,
    pub url: Option<String>,
    pub target: Option<String>,
}

impl Default for TextFormat {
    fn default() -> Self {
        Self {
            font: None,
            size: None,
            color: None,
            align: None,
            bold: None,
            italic: None,
            underline: None,
            left_margin: None,
            right_margin: None,
            indent: None,
            block_indent: None,
            kerning: None,
            leading: None,
            letter_spacing: None,
            tab_stops: None,
            bullet: None,
            url: None,
            target: None,
        }
    }
}

fn getstr_from_avm1_object<'gc>(
    object: Object<'gc>,
    name: &str,
    avm1: &mut Avm1<'gc>,
    uc: &mut UpdateContext<'_, 'gc, '_>,
) -> Result<Option<String>, crate::avm1::error::Error> {
    Ok(match object.get(name, avm1, uc)? {
        Value::Undefined => None,
        Value::Null => None,
        v => Some(v.coerce_to_string(avm1, uc)?.to_string()),
    })
}

fn getfloat_from_avm1_object<'gc>(
    object: Object<'gc>,
    name: &str,
    avm1: &mut Avm1<'gc>,
    uc: &mut UpdateContext<'_, 'gc, '_>,
) -> Result<Option<f64>, crate::avm1::error::Error> {
    Ok(match object.get(name, avm1, uc)? {
        Value::Undefined => None,
        Value::Null => None,
        v => Some(v.coerce_to_f64(avm1, uc)?),
    })
}

fn getbool_from_avm1_object<'gc>(
    object: Object<'gc>,
    name: &str,
    avm1: &mut Avm1<'gc>,
    uc: &mut UpdateContext<'_, 'gc, '_>,
) -> Result<Option<bool>, crate::avm1::error::Error> {
    Ok(match object.get(name, avm1, uc)? {
        Value::Undefined => None,
        Value::Null => None,
        v => Some(v.as_bool(avm1.current_swf_version())),
    })
}

impl TextFormat {
    /// Construct a `TextFormat` from an `EditText`'s SWF tag.
    ///
    /// This requires an `UpdateContext` as we will need to retrieve some font
    /// information from the actually-referenced font.
    pub fn from_swf_tag<'gc>(
        et: swf::EditText,
        swf_movie: Arc<SwfMovie>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Self {
        let movie_library = context.library.library_for_movie_mut(swf_movie);

        //TODO: How do we represent fonts referenced by character rather than
        //by linkage name?
        let font = et.font_class_name;
        let align = et.layout.clone().map(|l| l.align);
        let left_margin = et.layout.clone().map(|l| l.left_margin.to_pixels());
        let right_margin = et.layout.clone().map(|l| l.right_margin.to_pixels());
        let indent = et.layout.clone().map(|l| l.indent.to_pixels());
        let leading = et.layout.map(|l| l.leading.to_pixels());

        Self {
            font,
            size: et.height.map(|h| h.to_pixels()),
            color: et.color,
            align,
            bold: None,      // TODO: Resolve a font and pull this from that font
            italic: None,    // TODO: Resolve a font and pull this from that font
            underline: None, // TODO: Resolve a font and pull this from that font
            left_margin,
            right_margin,
            indent,
            block_indent: Some(0.0), // TODO: This isn't specified by the tag itself
            kerning: Some(true),     // TODO: this isn't specified by the tag itself
            leading,
            letter_spacing: Some(0.0), // TODO: This isn't specified by the tag itself
            tab_stops: Some(vec![]),   // TODO: Are there default tab stops?
            bullet: Some(false),       // TODO: Default tab stops?

            // TODO: These are probably empty strings by default
            url: Some("".to_string()),
            target: Some("".to_string()),
        }
    }

    /// Construct a `TextFormat` from an object that is
    pub fn from_avm1_object<'gc>(
        object1: Object<'gc>,
        avm1: &mut Avm1<'gc>,
        uc: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<Self, crate::avm1::error::Error> {
        Ok(Self {
            font: getstr_from_avm1_object(object1, "font", avm1, uc)?,
            size: getfloat_from_avm1_object(object1, "size", avm1, uc)?,
            color: getfloat_from_avm1_object(object1, "color", avm1, uc)?
                .map(|v| swf::Color::from_rgb(v as u32, 0xFF)),
            align: getstr_from_avm1_object(object1, "align", avm1, uc)?.and_then(|v| {
                match v.to_lowercase().as_str() {
                    "left" => Some(swf::TextAlign::Left),
                    "center" => Some(swf::TextAlign::Center),
                    "right" => Some(swf::TextAlign::Right),
                    "justify" => Some(swf::TextAlign::Justify),
                    _ => None,
                }
            }),
            bold: getbool_from_avm1_object(object1, "bold", avm1, uc)?,
            italic: getbool_from_avm1_object(object1, "italic", avm1, uc)?,
            underline: getbool_from_avm1_object(object1, "underline", avm1, uc)?,
            left_margin: getfloat_from_avm1_object(object1, "leftMargin", avm1, uc)?,
            right_margin: getfloat_from_avm1_object(object1, "rightMargin", avm1, uc)?,
            indent: getfloat_from_avm1_object(object1, "indent", avm1, uc)?,
            block_indent: getfloat_from_avm1_object(object1, "blockIndent", avm1, uc)?,
            kerning: getbool_from_avm1_object(object1, "kerning", avm1, uc)?,
            leading: getfloat_from_avm1_object(object1, "leading", avm1, uc)?,
            letter_spacing: getfloat_from_avm1_object(object1, "letterSpacing", avm1, uc)?,
            tab_stops: None,
            bullet: getbool_from_avm1_object(object1, "bullet", avm1, uc)?,
            url: getstr_from_avm1_object(object1, "url", avm1, uc)?,
            target: getstr_from_avm1_object(object1, "target", avm1, uc)?,
        })
    }

    /// Extract text format parameters from presentational markup.
    ///
    /// This assumes the "legacy" HTML path that only supports a handful of
    /// elements. The "stylesheet" HTML path will also require CSS style
    /// calculation for each node, followed by style conversion.
    pub fn from_presentational_markup(node: XMLNode<'_>) -> Self {
        match node.tag_name() {
            Some(name) if name == XMLName::from_str("p") => {
                let mut tf = TextFormat::default();

                match node.attribute_value(&XMLName::from_str("align")).as_deref() {
                    Some("left") => tf.align = Some(swf::TextAlign::Left),
                    Some("center") => tf.align = Some(swf::TextAlign::Center),
                    Some("right") => tf.align = Some(swf::TextAlign::Right),
                    _ => {}
                }

                tf
            }
            Some(name) if name == XMLName::from_str("a") => {
                let mut tf = TextFormat::default();

                if let Some(href) = node.attribute_value(&XMLName::from_str("href")) {
                    tf.url = Some(href);
                }

                if let Some(target) = node.attribute_value(&XMLName::from_str("target")) {
                    tf.target = Some(target);
                }

                tf
            }
            Some(name) if name == XMLName::from_str("font") => {
                let mut tf = TextFormat::default();

                if let Some(face) = node.attribute_value(&XMLName::from_str("face")) {
                    tf.font = Some(face);
                }

                if let Some(size) = node.attribute_value(&XMLName::from_str("size")) {
                    tf.size = size.parse().ok();
                }

                if let Some(color) = node.attribute_value(&XMLName::from_str("color")) {
                    if color.starts_with('#') {
                        let rval = color.get(1..3).and_then(|v| u8::from_str_radix(v, 16).ok());
                        let gval = color.get(3..5).and_then(|v| u8::from_str_radix(v, 16).ok());
                        let bval = color.get(5..7).and_then(|v| u8::from_str_radix(v, 16).ok());

                        if let (Some(r), Some(g), Some(b)) = (rval, gval, bval) {
                            tf.color = Some(swf::Color { r, g, b, a: 255 });
                        }
                    }
                }

                tf
            }
            Some(name) if name == XMLName::from_str("b") => {
                let mut tf = TextFormat::default();

                tf.bold = Some(true);

                tf
            }
            Some(name) if name == XMLName::from_str("i") => {
                let mut tf = TextFormat::default();

                tf.italic = Some(true);

                tf
            }
            Some(name) if name == XMLName::from_str("u") => {
                let mut tf = TextFormat::default();

                tf.underline = Some(true);

                tf
            }
            Some(name) if name == XMLName::from_str("li") => {
                let mut tf = TextFormat::default();

                // TODO: Does `bullet` indicate the start of a new bullet in
                // spans, or does it just mean each line gets a bullet, and
                // this tag shouldn't touch formatting?
                tf.bullet = Some(true);

                tf
            }
            Some(name) if name == XMLName::from_str("textformat") => {
                let mut tf = TextFormat::default();

                //TODO: Spec says these are all in twips. That doesn't seem to
                //match Flash 8.
                if let Some(left_margin) = node.attribute_value(&XMLName::from_str("leftmargin")) {
                    tf.left_margin = left_margin.parse().ok();
                }

                if let Some(right_margin) = node.attribute_value(&XMLName::from_str("rightmargin"))
                {
                    tf.right_margin = right_margin.parse().ok();
                }

                if let Some(indent) = node.attribute_value(&XMLName::from_str("indent")) {
                    tf.indent = indent.parse().ok();
                }

                if let Some(blockindent) = node.attribute_value(&XMLName::from_str("blockindent")) {
                    tf.block_indent = blockindent.parse().ok();
                }

                if let Some(leading) = node.attribute_value(&XMLName::from_str("leading")) {
                    tf.leading = leading.parse().ok();
                }

                if let Some(tabstops) = node.attribute_value(&XMLName::from_str("tabstops")) {
                    tf.tab_stops = Some(
                        tabstops
                            .split(',')
                            .filter_map(|v| v.trim().parse().ok())
                            .collect(),
                    );
                }

                tf
            }
            _ => TextFormat::default(),
        }
    }

    /// Construct a `TextFormat` AVM1 object from this text format object.
    pub fn as_avm1_object<'gc>(
        &self,
        avm1: &mut Avm1<'gc>,
        uc: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<Object<'gc>, crate::avm1::error::Error> {
        let object = ScriptObject::object(uc.gc_context, Some(avm1.prototypes().text_format));

        object.set(
            "font",
            self.font.clone().map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "size",
            self.size.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "color",
            self.color
                .clone()
                .map(|v| (((v.r as u32) << 16) + ((v.g as u32) << 8) + v.b as u32).into())
                .unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "align",
            self.align
                .map(|v| {
                    match v {
                        swf::TextAlign::Left => "left",
                        swf::TextAlign::Center => "center",
                        swf::TextAlign::Right => "right",
                        swf::TextAlign::Justify => "justify",
                    }
                    .into()
                })
                .unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "bold",
            self.bold.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "italic",
            self.italic.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "underline",
            self.underline.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "leftMargin",
            self.left_margin.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "rightMargin",
            self.right_margin.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "indent",
            self.indent.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "blockIndent",
            self.block_indent.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "kerning",
            self.kerning.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "leading",
            self.leading.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "letterSpacing",
            self.letter_spacing.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "bullet",
            self.bullet.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "url",
            self.url.clone().map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "target",
            self.target.clone().map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;

        Ok(object.into())
    }

    /// Given two text formats, construct a new `TextFormat` where only
    /// matching properties between the two formats are defined.
    pub fn merge_matching_properties(self, rhs: TextFormat) -> Self {
        TextFormat {
            font: if self.font == rhs.font {
                self.font
            } else {
                None
            },
            size: if self.size == rhs.size {
                self.size
            } else {
                None
            },
            color: if self.color == rhs.color {
                self.color
            } else {
                None
            },
            align: if self.align == rhs.align {
                self.align
            } else {
                None
            },
            bold: if self.bold == rhs.bold {
                self.bold
            } else {
                None
            },
            italic: if self.italic == rhs.italic {
                self.italic
            } else {
                None
            },
            underline: if self.underline == rhs.underline {
                self.underline
            } else {
                None
            },
            left_margin: if self.left_margin == rhs.left_margin {
                self.left_margin
            } else {
                None
            },
            right_margin: if self.right_margin == rhs.right_margin {
                self.right_margin
            } else {
                None
            },
            indent: if self.indent == rhs.indent {
                self.indent
            } else {
                None
            },
            block_indent: if self.block_indent == rhs.block_indent {
                self.block_indent
            } else {
                None
            },
            kerning: if self.kerning == rhs.kerning {
                self.kerning
            } else {
                None
            },
            leading: if self.leading == rhs.leading {
                self.leading
            } else {
                None
            },
            letter_spacing: if self.letter_spacing == rhs.letter_spacing {
                self.letter_spacing
            } else {
                None
            },
            tab_stops: if self.tab_stops == rhs.tab_stops {
                self.tab_stops
            } else {
                None
            },
            bullet: if self.bullet == rhs.bullet {
                self.bullet
            } else {
                None
            },
            url: if self.url == rhs.url { self.url } else { None },
            target: if self.target == rhs.target {
                self.target
            } else {
                None
            },
        }
    }
}

/// Represents the application of a `TextFormat` to a particular text span.
///
/// The actual string data is not stored here; a `TextSpan` is meaningless
/// without it's underlying string content. Furthermore, the start position
/// within the string is implicit in the sum of all previous text span's
/// lengths. See `TextSpans` for more information.
///
/// This struct also contains a resolved version of the `TextFormat` structure
/// listed above.
#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub struct TextSpan {
    /// How many characters are subsumed by this text span.
    ///
    /// This value must not cause the resulting set of text spans to exceed the
    /// length of the underlying source string.
    span_length: usize,

    font: String,
    size: f64,
    color: swf::Color,
    align: swf::TextAlign,
    bold: bool,
    italic: bool,
    underline: bool,
    left_margin: f64,
    right_margin: f64,
    indent: f64,
    block_indent: f64,
    kerning: bool,
    leading: f64,
    letter_spacing: f64,
    tab_stops: Vec<f64>,
    bullet: bool,
    url: String,
    target: String,
}

impl Default for TextSpan {
    fn default() -> Self {
        Self {
            span_length: 0,
            font: "".to_string(),
            size: 12.0,
            color: swf::Color {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
            align: swf::TextAlign::Left,
            bold: false,
            italic: false,
            underline: false,
            left_margin: 0.0,
            right_margin: 0.0,
            indent: 0.0,
            block_indent: 0.0,
            kerning: false,
            leading: 0.0,
            letter_spacing: 0.0,
            tab_stops: vec![],
            bullet: false,
            url: "".to_string(),
            target: "".to_string(),
        }
    }
}

impl TextSpan {
    pub fn with_length(length: usize) -> Self {
        let mut data = Self::default();

        data.span_length = length;

        data
    }

    pub fn with_length_and_format(length: usize, tf: TextFormat) -> Self {
        let mut data = Self::default();

        data.span_length = length;
        data.set_text_format(&tf);

        data
    }

    /// Split the text span in two at a particular point relative to the
    /// current text span's start.
    ///
    /// The second span is returned and should be inserted into the list of
    /// text spans appropriately. The first text span is changed in-line.
    ///
    /// If the split point exceeds the size of the current span, then no span
    /// will be returned and no change will be made to the existing span.
    fn split_at(&mut self, split_point: usize) -> Option<Self> {
        if self.span_length <= split_point || split_point == 0 {
            return None;
        }

        let mut new_span = self.clone();
        new_span.span_length = self.span_length - split_point;
        self.span_length = split_point;

        Some(new_span)
    }

    /// Determine if this and another span have identical text formats.
    ///
    /// It is assumed that the two text spans being considered are adjacent;
    /// and we have no way of checking, so this function doesn't check that.
    #[allow(clippy::float_cmp)]
    fn can_merge(&self, rhs: &Self) -> bool {
        self.font == rhs.font
            && self.size == rhs.size
            && self.color == rhs.color
            && self.align == rhs.align
            && self.bold == rhs.bold
            && self.italic == rhs.italic
            && self.underline == rhs.underline
            && self.left_margin == rhs.left_margin
            && self.right_margin == rhs.right_margin
            && self.indent == rhs.indent
            && self.block_indent == rhs.block_indent
            && self.kerning == rhs.kerning
            && self.leading == rhs.leading
            && self.letter_spacing == rhs.letter_spacing
            && self.tab_stops == rhs.tab_stops
            && self.bullet == rhs.bullet
            && self.url == rhs.url
            && self.target == rhs.target
    }

    /// Merge two spans together.
    ///
    /// This function assumes the two spans are adjacent; if they are not, this
    /// will break invariants of the text span system.
    ///
    /// If the spans do not have identical text formatting, this function will
    /// refuse to merge them (see `can_merge`) and return the original `rhs`
    /// span. If it consumes the span, and yields None, then the merge
    /// completed successfully.
    fn merge(&mut self, rhs: Self) -> Option<Self> {
        if !self.can_merge(&rhs) {
            return Some(rhs);
        }

        self.span_length += rhs.span_length;

        None
    }

    /// Apply a text format to this text span.
    ///
    /// Properties marked `None` on the `TextFormat` will remain unchanged.
    fn set_text_format(&mut self, tf: &TextFormat) {
        if let Some(font) = &tf.font {
            self.font = font.clone();
        }

        if let Some(size) = &tf.size {
            self.size = *size;
        }

        if let Some(color) = &tf.color {
            self.color = color.clone();
        }

        if let Some(align) = &tf.align {
            self.align = *align;
        }

        if let Some(bold) = &tf.bold {
            self.bold = *bold;
        }

        if let Some(italic) = &tf.italic {
            self.italic = *italic;
        }

        if let Some(underline) = &tf.underline {
            self.underline = *underline;
        }

        if let Some(left_margin) = &tf.left_margin {
            self.left_margin = *left_margin;
        }

        if let Some(right_margin) = &tf.right_margin {
            self.right_margin = *right_margin;
        }

        if let Some(indent) = &tf.indent {
            self.indent = *indent;
        }

        if let Some(block_indent) = &tf.block_indent {
            self.block_indent = *block_indent;
        }

        if let Some(kerning) = &tf.kerning {
            self.kerning = *kerning;
        }

        if let Some(leading) = &tf.leading {
            self.leading = *leading;
        }

        if let Some(letter_spacing) = &tf.letter_spacing {
            self.letter_spacing = *letter_spacing;
        }

        if let Some(tab_stops) = &tf.tab_stops {
            self.tab_stops = tab_stops.clone();
        }

        if let Some(bullet) = &tf.bullet {
            self.bullet = *bullet;
        }

        if let Some(url) = &tf.url {
            self.url = url.clone();
        }

        if let Some(target) = &tf.target {
            self.target = target.clone();
        }
    }

    fn get_text_format(&self) -> TextFormat {
        TextFormat {
            font: Some(self.font.clone()),
            size: Some(self.size),
            color: Some(self.color.clone()),
            align: Some(self.align),
            bold: Some(self.bold),
            italic: Some(self.italic),
            underline: Some(self.underline),
            left_margin: Some(self.left_margin),
            right_margin: Some(self.right_margin),
            indent: Some(self.indent),
            block_indent: Some(self.block_indent),
            kerning: Some(self.kerning),
            leading: Some(self.leading),
            letter_spacing: Some(self.letter_spacing),
            tab_stops: Some(self.tab_stops.clone()),
            bullet: Some(self.bold),
            url: Some(self.url.clone()),
            target: Some(self.target.clone()),
        }
    }
}

/// Struct which contains text formatted by `TextSpan`s.
#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub struct FormatSpans {
    text: String,
    spans: Vec<TextSpan>,
    default_format: TextFormat,
}

impl FormatSpans {
    pub fn from_str_and_format(text: &str, default_format: TextFormat) -> Self {
        let mut span = TextSpan::with_length(text.len());

        span.set_text_format(&default_format);

        FormatSpans {
            text: text.to_string(),
            spans: vec![span],
            default_format,
        }
    }

    /// Construct a format span from it's raw parts.
    pub fn from_str_and_spans(text: &str, spans: &[TextSpan]) -> Self {
        FormatSpans {
            text: text.to_string(),
            spans: spans.to_vec(),
            default_format: Default::default(),
        }
    }

    pub fn default_format(&self) -> &TextFormat {
        &self.default_format
    }

    pub fn set_default_format(&mut self, tf: TextFormat) {
        self.default_format = tf;
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    /// Find the index of the span that covers a given search position.
    ///
    /// This function returns both the index of the span which covers the
    /// search position, but how far into the span it's position is.
    ///
    /// The index returned from this function is not valid across calls which
    /// mutate spans.
    pub fn resolve_position_as_span(&self, search_pos: usize) -> Option<(usize, usize)> {
        let mut position = 0;

        for (index, span) in self.spans.iter().enumerate() {
            if search_pos < position + span.span_length {
                return Some((index, search_pos - position));
            }

            position += span.span_length;
        }

        None
    }

    /// Create a text-span break at a particular position, if one does not
    /// already exist.
    ///
    /// If `search_pos` is out of bounds for the underlying set of spans, then
    /// this function returns `None`.
    ///
    /// The returned index refers to the index of the newly-created span at
    /// `search_pos`. It will be invalidated if another span break is created
    /// before `search_pos` or if the spans are normalized. If you need to
    /// create multiple span breaks, you must either:
    ///
    ///  * Always create spans in order of increasing `search_pos`.
    ///  * Discard the values returned by this function and redundantly resolve
    ///    each span again once all breaks are completed.
    pub fn ensure_span_break_at(&mut self, search_pos: usize) -> Option<usize> {
        if let Some((first_span_pos, break_index)) = self.resolve_position_as_span(search_pos) {
            if break_index == 0 {
                return Some(first_span_pos);
            }

            let first_span = self.spans.get_mut(first_span_pos).unwrap();
            let mut second_span = first_span.clone();
            second_span.span_length = first_span.span_length - break_index;
            first_span.span_length = break_index;

            self.spans.insert(first_span_pos + 1, second_span);

            Some(first_span_pos + 1)
        } else {
            None
        }
    }

    /// Retrieve the range of spans that encompass the text range [from, to).
    ///
    /// The range returned by this function is the clopen set [span_from,
    /// span_to) ready to be sliced as `&spans[span_from..span_to]`.
    ///
    /// The boundaries yielded by this function may extend beyond the text
    /// range, but will always at least encompass all of the text. To get an
    /// exact text range, you must first call `ensure_span_break_at` for both
    /// `from` and `to`.
    ///
    /// The indexes returned from this function is not valid across calls which
    /// mutate spans.
    pub fn get_span_boundaries(&self, from: usize, to: usize) -> (usize, usize) {
        let start_pos = self.resolve_position_as_span(from).unwrap_or((0, 0)).0;
        let end_pos = min(
            self.resolve_position_as_span(to.saturating_sub(1))
                .map(|(pos, i)| (pos.saturating_add(1), i))
                .unwrap_or_else(|| (self.spans.len(), 0))
                .0,
            self.spans.len(),
        );

        (start_pos, end_pos)
    }

    /// Adjust the format spans in several ways to ensure that other function
    /// invariants are upheld.
    ///
    /// The particular variants that `normalize` attempts to uphold are:
    ///
    ///  * The length implied by the list of text spans must match the length
    ///    of the string they are formatting.
    ///  * Adjacent text spans contain different text formats. (Stated
    ///    contrapositively, `normalize` attempts to merge text spans with
    ///    identical formatting.)
    ///  * All text spans have non-zero length, unless the associated string is
    ///    legitimately empty, in which case the span list should be a *single*
    ///    null-length span.
    ///  * There is always at least one text span.
    ///
    /// This function should always be called after mutating text spans in such
    /// a way that might violate the above-mentioned invariants.
    pub fn normalize(&mut self) {
        let mut span_length = 0;
        for span in self.spans.iter() {
            span_length += span.span_length;
        }

        match span_length.cmp(&self.text.len()) {
            Ordering::Less => self.spans.push(TextSpan::with_length_and_format(
                self.text.len() - span_length,
                self.default_format.clone(),
            )),
            Ordering::Greater => {
                let mut deficiency = span_length - self.text.len();
                while deficiency > 0 && !self.spans.is_empty() {
                    let removed_length = {
                        let last = self.spans.last_mut().unwrap();
                        if last.span_length > deficiency {
                            last.span_length -= deficiency;
                            break;
                        } else {
                            last.span_length
                        }
                    };

                    self.spans.pop();
                    deficiency -= removed_length;
                }
            }
            Ordering::Equal => {}
        }

        // Remove leading null-length spans. The null-length span removal in
        // the loop below cannot cope with the situation where the first span
        // is null-length, so we ensure it always gets a span with valid
        // length.
        while self
            .spans
            .get(0)
            .map(|span| span.span_length == 0)
            .unwrap_or(false)
        {
            self.spans.remove(0);
        }

        let mut i = 0;
        while i < self.spans.len().saturating_sub(1) {
            let remove_next = {
                let spans = self.spans.get_mut(i..i + 2).unwrap();

                if spans[0].can_merge(&spans[1]) || spans[1].span_length == 0 {
                    spans[0].span_length += spans[1].span_length;
                    true
                } else {
                    false
                }
            };

            if remove_next {
                self.spans.remove(i + 1);
            } else {
                i += 1;
            }
        }

        // Null span removal can possibly cause the span list to become empty.
        // If that happens, then insert a new span. We don't care if it's a
        // null span at this point.
        if self.spans.is_empty() {
            self.spans.push(TextSpan::with_length_and_format(
                self.text.len(),
                self.default_format.clone(),
            ));
        }
    }

    /// Retrieve a text format covering all of the properties applied to text
    /// from the start index to the end index.
    ///
    /// Any property that differs between spans of text will result in a `None`
    /// in the final text format.
    pub fn get_text_format(&self, from: usize, to: usize) -> TextFormat {
        let (start_pos, end_pos) = self.get_span_boundaries(from, to);
        let mut merged_fmt = if let Some(start_span) = self.spans.get(start_pos) {
            start_span.get_text_format()
        } else {
            return Default::default();
        };

        if let Some(spans) = self.spans.get(start_pos + 1..end_pos) {
            for span in spans {
                merged_fmt = merged_fmt.merge_matching_properties(span.get_text_format());
            }
        }

        merged_fmt
    }

    /// Change some portion of the text to have a particular set of text
    /// attributes.
    pub fn set_text_format(&mut self, from: usize, to: usize, fmt: &TextFormat) {
        self.ensure_span_break_at(from);
        self.ensure_span_break_at(to);

        let (start_pos, end_pos) = self.get_span_boundaries(from, to);

        if let Some(spans) = self.spans.get_mut(start_pos..end_pos) {
            for span in spans {
                span.set_text_format(fmt);
            }
        }

        self.normalize();
    }

    /// Replace the text in the range [from, to) with the contents of `with`.
    ///
    /// Attempts to remove degenerate ranges (e.g. [5, 2)) will fail silently.
    ///
    /// Text span formatting will be adjusted to match: specifically, the spans
    /// corresponding to the range will be removed and replaced with a single
    /// span for the newly inserted text. It's formatting will be determined by
    /// either the formatting of the last span in the range, or if the range
    /// extends beyond the end of the field, the default text format.
    ///
    /// (The text formatting behavior has been confirmed by manual testing with
    /// Flash Player 8.)
    pub fn replace_text(&mut self, from: usize, to: usize, with: &str) {
        if to < from {
            return;
        }

        if from < self.text.len() {
            self.ensure_span_break_at(from);
            self.ensure_span_break_at(to);

            let (start_pos, end_pos) = self.get_span_boundaries(from, to);
            let mut new_tf = self.default_format.clone();
            if let Some(span) = self.spans.get(end_pos) {
                new_tf = span.get_text_format();
            }

            self.spans.drain(start_pos..end_pos);
            self.spans.insert(
                start_pos,
                TextSpan::with_length_and_format(with.len(), new_tf),
            );
        } else {
            self.spans.push(TextSpan::with_length_and_format(
                with.len(),
                self.default_format.clone(),
            ));
        }

        let mut new_string = String::new();
        if let Some(text) = self.text.get(0..from) {
            new_string.push_str(text);
        } else {
            // `get` will fail if `from` exceeds the bounds of the text, rather
            // than just giving all of it to us. In that case, we append the
            // entire string.
            new_string.push_str(&self.text);
        }

        new_string.push_str(with);

        if let Some(text) = self.text.get(to..) {
            new_string.push_str(text);
        }

        self.text = new_string;

        self.normalize();
    }

    /// Lower an HTML tree into text-span representation.
    ///
    /// This is the "legacy" implementation of this process: it only looks for
    /// a handful of presentational attributes in the HTML tree to generate
    /// styling. There's also a `lower_from_css` that respects both
    /// presentational markup and CSS stylesheets.
    pub fn lower_from_html<'gc>(&mut self, tree: XMLDocument<'gc>) {
        let mut format_stack = vec![];

        for step in tree.as_node().walk().unwrap() {
            match step {
                Step::In(node) => format_stack.push(TextFormat::from_presentational_markup(node)),
                Step::Around(node) if node.is_text() => {
                    //TODO: Append a text node...
                }
                Step::Out(_) => {
                    format_stack.pop();
                }
                _ => {}
            };
        }
    }
}
