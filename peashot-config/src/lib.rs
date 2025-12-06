use std::str::FromStr;

mod kdl;

use annotate_snippets::level::ERROR;

#[derive(strum::VariantNames, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
enum Toggle {
    On,
    Off,
}

struct Config {
    size_indicator: Toggle,
    selection_icons: Toggle,
    theme: Theme,
    keymap: Keymap,
}

#[test]
fn parse_kdl() {
    let kdl_src = include_str!("config.kdl");
    let kdl = match kdl_src.parse::<::kdl::KdlDocument>() {
        Ok(kdl) => kdl,
        Err(err) => {
            let diagnostics = err
                .diagnostics
                .into_iter()
                .map(|diagnostic| {
                    let level = match diagnostic.severity {
                        miette::Severity::Advice => annotate_snippets::Level::INFO,
                        miette::Severity::Warning => annotate_snippets::Level::WARNING,
                        miette::Severity::Error => annotate_snippets::Level::ERROR,
                    };
                    let error = level
                        .primary_title(
                            diagnostic
                                .message
                                .unwrap_or_else(|| "Failed to parse KDL".to_string()),
                        )
                        .element(
                            annotate_snippets::Snippet::source(kdl_src)
                                .path("config.kdl")
                                .annotation({
                                    let annotation = annotate_snippets::AnnotationKind::Primary
                                        .span(
                                            diagnostic.span.offset()
                                                ..diagnostic.span.offset() + diagnostic.span.len(),
                                        );

                                    if let Some(label) = diagnostic.label {
                                        annotation.label(label)
                                    } else {
                                        annotation
                                    }
                                }),
                        );
                    if let Some(help) = diagnostic.help {
                        error.element(annotate_snippets::Element::Message(
                            annotate_snippets::Level::HELP.message(help),
                        ))
                    } else {
                        error
                    }
                })
                .collect::<Vec<_>>();

            let renderer = annotate_snippets::Renderer::styled()
                .decor_style(annotate_snippets::renderer::DecorStyle::Unicode);
            let render = renderer.render(&diagnostics);
            eprintln!("{render}");
            panic!();
        },
    };

    struct Error {
        message: annotate_snippets::Title<'static>,
        span: miette::SourceSpan,
    }

    let mut errors = Vec::new();

    macro_rules! error {
        ($span:expr, $($tt:tt)*) => {
            errors.push(Error {
                message: annotate_snippets::Level::ERROR.primary_title(format!($($tt)*)),
                span: $span
            });
        };
    }

    for node in kdl {
        if let Some(ty) = node.ty() {
            error!(ty.span(), "unexpected type annotation");
        }
        match node.name().value() {
            "keymap" => {},
        }
    }

    panic!();
}
