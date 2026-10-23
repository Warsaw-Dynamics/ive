use std::{borrow::Cow,ops::Range,mem::size_of};
#[derive(Debug)]
enum Language { Mermaid, PlantUml }
enum RenderError {
    UnsupportedLanguage(Language),
    InvalidOptions(String),
    SourceLimit { actual: usize, limit: usize },
    ExternalResource(String),
    UnsupportedLayout(String),
    UnsupportedFeature(String),
    InvalidSource(String),
    Cancelled(String),
    Backend(String),
    UnsafeSvg(String),
}

enum Before { Syntax(&'static str), Unsupported(&'static str), Terminal(RenderError) }
enum After { Syntax(Cow<'static,str>), Unsupported(Cow<'static,str>), Terminal(RenderError) }
struct OldFailure {span:Range<usize>,kind:Before}
struct NewFailure {span:Range<usize>,kind:After}
fn main(){println!("RenderError={} BeforeKind={} AfterKind={} BeforeFailure={} AfterFailure={}",size_of::<RenderError>(),size_of::<Before>(),size_of::<After>(),size_of::<OldFailure>(),size_of::<NewFailure>());}
