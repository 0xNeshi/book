extern crate trpl; // required for mdbook test

use trpl::{Either, Html};

fn main() {
    // TODO: we'll add this next!
}

async fn page_title_for(url: &str) -> Option<String> {
    let response = trpl::get(url);
    let response_text = response.text();
    Html::parse(&response_text)
        .select_first("title")
        .map(|title| title.inner_html())
}
