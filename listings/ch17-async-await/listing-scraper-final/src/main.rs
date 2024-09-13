extern crate trpl; // required for mdbook test

use trpl::{Either, Html};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    trpl::run(async {
        let first = page_title_for(&args[1]);
        let second = page_title_for(&args[2]);

        let winning_page = match trpl::race(first, second).await {
            Either::Left(left) => left,
            Either::Right(right) => right,
        };

        println!("The winner was {winning_page:?}");
    })
}

async fn page_title_for(url: &str) -> Option<String> {
    let text = trpl::get(url).await.text().await;
    Html::parse(&text)
        .select_first("title")
        .map(|title| title.inner_html())
}
