use std::{cell::RefCell, rc::Rc};

use crate::{
    database::{Carta, DatabaseCache, DATABASE, DATABASE_CACHE},
    state::ClientState,
    tree::TreeBranch,
};

use anyhow::{anyhow, Context};
use fix_fn::fix_fn;
use twinstar::{document::HeadingLevel, Document};

pub fn display_field<'a>(field: &'a Option<String>, sentinel: &'a str) -> &'a str {
    field.as_deref().unwrap_or(sentinel).trim_end()
}

/// Fetch cartas page UI
pub fn handle_viewing_carta(client: &mut ClientState, uuid: String) -> anyhow::Result<String> {
    let mut document = Document::new();

    let carta = DatabaseCache::get_or_else(&DATABASE_CACHE.carta, &uuid, &|| {
        let mut database_guard = DATABASE
            .lock()
            .map_err(|_| anyhow!("failed to lock database mutex"))?;
        database_guard.fetch_carta_uuid(&uuid)
    })?;
    let mut database_guard = DATABASE
        .lock()
        .map_err(|_| anyhow!("failed to lock database mutex"))?;
    let carta_tree = database_guard
        .fetch_carta_tree(carta.id)
        .context("fetching carta tree")?;

    // Display carta
    document.add_heading(
        HeadingLevel::H3,
        format!(
            "=== {from} - {title}",
            from = display_field(&carta.sender, &client.lang.write_from_sentinel),
            title = display_field(&carta.title, &client.lang.write_untitled_sentinel)
        ),
    );
    for line in carta.content.split('\n') {
        document.add_text(line);
    }
    document.add_heading(HeadingLevel::H3, "===");

    // Display reply tree
    let document_ref = RefCell::new(document);
    #[allow(clippy::unused_unit)] // fix_fn needs a return type
    let reply_tree = fix_fn!(
        |reply_tree, indent: usize, tree: Rc<TreeBranch<Carta>>| -> () {
            let current = tree.node.uuid == uuid;
            document_ref.borrow_mut().add_link(
                format!("read-{uuid}", uuid = &tree.node.uuid).as_str(),
                format!(
                    "{indent} {from} - {title}",
                    indent = if !current { "-- " } else { "++ " }.repeat(indent),
                    from = display_field(&tree.node.sender, &client.lang.write_from_sentinel),
                    title = display_field(&tree.node.title, &client.lang.write_untitled_sentinel)
                ),
            );
            for child in tree.children.borrow().iter() {
                reply_tree(indent + 1, Rc::clone(child))
            }
        }
    );
    reply_tree(0, Rc::new(carta_tree));
    let mut document = document_ref.into_inner();

    document
        .add_blank_line()
        .add_link("fetch", &client.lang.write_return_link);

    Ok(document.to_string())
}
