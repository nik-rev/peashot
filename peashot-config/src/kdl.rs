use kdl::KdlDocument;
use kdl::KdlNode;
use kdl::KdlValue;

struct Error {
    message: annotate_snippets::Title<'static>,
    span: miette::SourceSpan,
}

impl Error {
    pub fn context(&mut self, message: impl Into<String>, span: miette::SourceSpan) -> &mut Self {
        // TODO
        self
    }
}

pub struct Errors {
    errors: Vec<Error>,
}

impl Errors {
    pub fn emit(&mut self, message: impl Into<String>, span: miette::SourceSpan) -> &mut Error {
        self.errors.push(Error {
            message: annotate_snippets::Level::ERROR.primary_title(message.into()),
            span,
        });
        self.errors.last_mut().expect("just pushed a new value")
    }
}

/// # On structs
///
/// - Each field must implement `KdlNodeCodec`
pub trait KdlDocumentCodec {
    fn decode(kdl: KdlDocument, errs: &mut Errors) -> Self;
    fn encode(self) -> KdlDocument;
}

/// # On structs
///
/// - If there is a `children` field, it must implement `KdlDocumentCodec`
/// - Each field with the `#[positional]` attribute must implement
///   `KdlArgumentCodec`
/// - All other fields must implement `KdlPropertyCodec`
pub trait KdlNodeCodec {
    fn decode(kdl: KdlNode, errs: &mut Errors) -> Self;
    fn encode(self) -> KdlNode;
}

pub trait KdlValueCodec {
    fn decode(kdl: KdlValue, errs: &mut Errors) -> Self;
    fn encode(self) -> KdlValue;
}
