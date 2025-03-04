use base_db::DocumentData;
use rowan::{ast::AstNode, TextRange};
use rustc_hash::FxHashMap;
use syntax::{
    bibtex::{self, HasName},
    latex,
};

use crate::util::cursor::CursorContext;

use super::{Indel, Params, RenameResult};

pub(super) fn prepare_rename<T>(context: &CursorContext<T>) -> Option<TextRange> {
    let (_, range) = context
        .find_citation_key_word()
        .or_else(|| context.find_entry_key())?;

    Some(range)
}

pub(super) fn rename<'a>(context: &CursorContext<'a, Params>) -> Option<RenameResult<'a>> {
    prepare_rename(context)?;
    let (key_text, _) = context
        .find_citation_key_word()
        .or_else(|| context.find_entry_key())?;

    let mut changes = FxHashMap::default();
    for document in &context.project.documents {
        match &document.data {
            DocumentData::Tex(data) => {
                let root = data.root_node();

                let edits: Vec<_> = root
                    .descendants()
                    .filter_map(latex::Citation::cast)
                    .filter_map(|citation| citation.key_list())
                    .flat_map(|keys| keys.keys())
                    .filter(|key| key.to_string() == key_text)
                    .map(|key| Indel {
                        delete: latex::small_range(&key),
                        insert: context.params.new_name.clone(),
                    })
                    .collect();

                changes.insert(*document, edits);
            }
            DocumentData::Bib(data) => {
                let root = data.root_node();
                let edits: Vec<_> = root
                    .descendants()
                    .filter_map(bibtex::Entry::cast)
                    .filter_map(|entry| entry.name_token())
                    .filter(|key| key.text() == key_text)
                    .map(|key| Indel {
                        delete: key.text_range(),
                        insert: context.params.new_name.clone(),
                    })
                    .collect();

                changes.insert(*document, edits);
            }
            DocumentData::Aux(_)
            | DocumentData::Log(_)
            | DocumentData::Root
            | DocumentData::Tectonic => {}
        };
    }

    Some(RenameResult { changes })
}
