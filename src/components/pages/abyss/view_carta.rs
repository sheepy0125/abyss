use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, UNIX_EPOCH},
};

use crate::{
    database::{Carta, DatabaseCache, DATABASE, DATABASE_CACHE},
    state::ClientState,
    tree::TreeBranch,
};

use anyhow::{anyhow, Context};
use chrono::{DateTime, Utc};
use fix_fn::fix_fn;
use twinstar::{document::HeadingLevel, Document};

pub fn display_field<'a>(field: &'a Option<String>, sentinel: &'a str) -> &'a str {
    field.as_deref().unwrap_or(sentinel).trim_end()
}
pub fn display_unix_timestamp(timestamp: u32) -> String {
    let timestamp = UNIX_EPOCH + Duration::from_secs(timestamp as _);
    let datetime = DateTime::<Utc>::from(timestamp);
    datetime.format("%Y-%m-%d %H:%M:%S GMT").to_string()
}

/// Fetch cartas page UI
pub fn handle_viewing_carta(client: &mut ClientState, uuid: String) -> anyhow::Result<String> {
    let mut document = Document::new();

    document
        .add_heading(HeadingLevel::H1, &client.lang.view_header)
        .add_blank_line();

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
            "{time} / {from} - {title}",
            time = display_unix_timestamp(carta.modification.unwrap_or(carta.creation) as _),
            from = display_field(&carta.sender, &client.lang.from_sentinel),
            title = display_field(&carta.title, &client.lang.untitled_sentinel)
        ),
    );
    for line in carta.content.split('\n') {
        document.add_preformatted(line);
    }
    document.add_heading(HeadingLevel::H3, "===");

    // Display reply tree
    document
        .add_blank_line()
        .add_heading(HeadingLevel::H3, &client.lang.view_replies_header);
    let document_ref = RefCell::new(document);
    #[allow(clippy::unused_unit)] // fix_fn needs a return type
    let reply_tree = fix_fn!(
        |reply_tree, indent: usize, tree: Rc<TreeBranch<Carta>>| -> () {
            let current = tree.node.uuid == uuid;
            document_ref.borrow_mut().add_link(
                format!("read-{uuid}", uuid = &tree.node.uuid).as_str(),
                format!(
                    "{indent}{from} - {title}",
                    indent = if !current { "- " } else { "+ " }.repeat(indent),
                    from = display_field(&tree.node.sender, &client.lang.from_sentinel),
                    title = display_field(&tree.node.title, &client.lang.untitled_sentinel)
                ),
            );
            for child in tree.children.borrow().iter() {
                reply_tree(indent + 1, Rc::clone(child))
            }
        }
    );
    reply_tree(1, Rc::new(carta_tree));
    let mut document = document_ref.into_inner();
    document.add_heading(HeadingLevel::H3, "===");

    if carta
        .user_id
        .is_some_and(|carta_id| carta_id == client.id() as _)
    {
        document
            .add_blank_line()
            .add_text(format!(
                "{text} {pin}{id}",
                text = &client.lang.delete_code_text,
                pin = carta.modification_code,
                id = carta.id
            ))
            .add_link("../delete", &client.lang.abyss_delete_link);
    }

    document
        .add_blank_line()
        .add_link(
            format!("reply-{uuid}").as_str(),
            &client.lang.view_add_reply_link,
        )
        .add_link(
            format!("report-{uuid}").as_str(),
            &client.lang.view_report_link,
        );

    document
        .add_blank_line()
        .add_link("fetch", &client.lang.return_link);

    Ok(document.to_string())
}
