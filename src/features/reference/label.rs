use cancellation::CancellationToken;
use lsp_types::{Location, ReferenceParams};

use crate::{
    features::cursor::CursorContext,
    syntax::{latex, CstNode},
    LineIndexExt,
};

pub fn find_label_references(
    context: &CursorContext<ReferenceParams>,
    cancellation_token: &CancellationToken,
    references: &mut Vec<Location>,
) -> Option<()> {
    cancellation_token.result().ok()?;

    let (name_text, _) = context
        .find_label_name_word()
        .or_else(|| context.find_label_name_command())?;

    for document in &context.request.subset.documents {
        if let Some(data) = document.data.as_latex() {
            for node in data.root.descendants() {
                cancellation_token.result().ok()?;

                if let Some(range) = latex::LabelDefinition::cast(node)
                    .filter(|_| context.request.params.context.include_declaration)
                    .and_then(|label| label.name())
                    .and_then(|name| name.word())
                    .filter(|name| name.text() == name_text)
                    .map(|name| document.line_index.line_col_lsp_range(name.text_range()))
                {
                    references.push(Location::new(document.uri.as_ref().clone().into(), range));
                }

                latex::LabelReference::cast(node)
                    .and_then(|label| label.name_list())
                    .into_iter()
                    .flat_map(|label| label.words())
                    .filter(|name| name.text() == name_text)
                    .map(|name| document.line_index.line_col_lsp_range(name.text_range()))
                    .for_each(|range| {
                        references.push(Location::new(document.uri.as_ref().clone().into(), range));
                    });

                if let Some(label) = latex::LabelReferenceRange::cast(node) {
                    if let Some(range) = label
                        .from()
                        .and_then(|name| name.word())
                        .filter(|name| name.text() == name_text)
                        .map(|name| document.line_index.line_col_lsp_range(name.text_range()))
                    {
                        references.push(Location::new(document.uri.as_ref().clone().into(), range));
                    }

                    if let Some(range) = label
                        .from()
                        .and_then(|name| name.word())
                        .filter(|name| name.text() == name_text)
                        .map(|name| document.line_index.line_col_lsp_range(name.text_range()))
                    {
                        references.push(Location::new(document.uri.as_ref().clone().into(), range));
                    }
                }
            }
        }
    }

    Some(())
}

#[cfg(test)]
mod tests {
    use lsp_types::Range;

    use crate::{features::testing::FeatureTester, RangeExt};

    use super::*;

    #[test]
    fn test_definition() {
        let tester = FeatureTester::builder()
            .files(vec![
                ("foo.tex", r#"\label{foo}"#),
                ("bar.tex", r#"\ref{foo}\input{foo.tex}"#),
            ])
            .main("foo.tex")
            .line(0)
            .character(8)
            .build();
        let uri = tester.uri("bar.tex");
        let mut actual_references = Vec::new();

        let request = tester.reference();
        let context = CursorContext::new(request);
        find_label_references(&context, CancellationToken::none(), &mut actual_references);

        let expected_references = vec![Location::new(
            uri.as_ref().clone().into(),
            Range::new_simple(0, 5, 0, 8),
        )];
        assert_eq!(actual_references, expected_references);
    }

    #[test]
    fn test_definition_include_declaration() {
        let tester = FeatureTester::builder()
            .files(vec![
                ("foo.tex", r#"\label{foo}\input{bar.tex}"#),
                ("bar.tex", r#"\ref{foo}"#),
            ])
            .main("foo.tex")
            .line(0)
            .character(9)
            .include_declaration(true)
            .build();
        let uri1 = tester.uri("foo.tex");
        let uri2 = tester.uri("bar.tex");
        let mut actual_references = Vec::new();

        let request = tester.reference();
        let context = CursorContext::new(request);
        find_label_references(&context, CancellationToken::none(), &mut actual_references);

        let expected_references = vec![
            Location::new(uri1.as_ref().clone().into(), Range::new_simple(0, 7, 0, 10)),
            Location::new(uri2.as_ref().clone().into(), Range::new_simple(0, 5, 0, 8)),
        ];
        assert_eq!(actual_references, expected_references);
    }

    #[test]
    fn test_reference() {
        let tester = FeatureTester::builder()
            .files(vec![
                ("foo.tex", r#"\label{foo}\input{bar.tex}"#),
                ("bar.tex", r#"\ref{foo}"#),
                ("baz.tex", r#"\ref{foo}\input{bar.tex}"#),
            ])
            .main("bar.tex")
            .line(0)
            .character(7)
            .build();
        let uri1 = tester.uri("bar.tex");
        let uri2 = tester.uri("baz.tex");
        let mut actual_references = Vec::new();

        let request = tester.reference();
        let context = CursorContext::new(request);
        find_label_references(&context, CancellationToken::none(), &mut actual_references);

        let expected_references = vec![
            Location::new(uri1.as_ref().clone().into(), Range::new_simple(0, 5, 0, 8)),
            Location::new(uri2.as_ref().clone().into(), Range::new_simple(0, 5, 0, 8)),
        ];
        assert_eq!(actual_references, expected_references);
    }

    #[test]
    fn test_reference_include_declaration() {
        let tester = FeatureTester::builder()
            .files(vec![
                ("foo.tex", r#"\label{foo}"#),
                ("bar.tex", r#"\ref{foo}\input{foo.tex}"#),
            ])
            .main("bar.tex")
            .line(0)
            .character(7)
            .include_declaration(true)
            .build();
        let uri1 = tester.uri("foo.tex");
        let uri2 = tester.uri("bar.tex");
        let mut actual_references = Vec::new();

        let request = tester.reference();
        let context = CursorContext::new(request);
        find_label_references(&context, CancellationToken::none(), &mut actual_references);

        let expected_references = vec![
            Location::new(uri2.as_ref().clone().into(), Range::new_simple(0, 5, 0, 8)),
            Location::new(uri1.as_ref().clone().into(), Range::new_simple(0, 7, 0, 10)),
        ];
        assert_eq!(actual_references, expected_references);
    }

    #[test]
    fn test_empty_latex_document() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "")])
            .main("main.tex")
            .line(0)
            .character(0)
            .build()
            .reference();
        let mut actual_references = Vec::new();

        let context = CursorContext::new(request);
        find_label_references(&context, CancellationToken::none(), &mut actual_references);

        assert!(actual_references.is_empty());
    }

    #[test]
    fn test_empty_bibtex_document() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "")])
            .main("main.bib")
            .line(0)
            .character(0)
            .build()
            .reference();
        let mut actual_references = Vec::new();

        let context = CursorContext::new(request);
        find_label_references(&context, CancellationToken::none(), &mut actual_references);

        assert!(actual_references.is_empty());
    }
}
