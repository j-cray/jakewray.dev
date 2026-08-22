//! Modular sections page definitions and re-exports.

pub use crate::pages::blog::PersonalBlogPage;
pub use crate::pages::creative::{CreativeWritingPage, MusicPage, PersonalPage, VisualArtPage};
pub use crate::pages::journalism::{render_article_card, JournalismArticlePage, JournalismPage};
pub use crate::pages::programming::ProgrammingPage;
pub use crate::utils::html::{
    bold_byline, extract_between, extract_body_preview, extract_printed_date, extract_subhead,
    format_cp_style, italicize_origin_line, linkify_images, process_article_content,
    replace_date_paragraph, starts_with_month, strip_tags,
};
pub use crate::utils::sorting::{
    extract_highlight_articles, get_article_sort_key, next_article_index, prev_article_index,
    sort_articles_newest_first, HIGHLIGHT_SLUGS,
};
