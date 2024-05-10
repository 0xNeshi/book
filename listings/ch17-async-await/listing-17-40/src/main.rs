use trpl::StreamExt;

fn main() {
    trpl::block_on(async {
        let values = 1..101;
        let iter = values.map(|n| n * 2);
        let stream = trpl::stream_from_iter(iter);

        // ANCHOR: filter
        let mut filtered =
            stream.filter(|value| value % 3 == 0 || value % 5 == 0);

        while let Some(value) = filtered.next().await {
            println!("The value was: {value}");
        }
        // ANCHOR_END: filter
    });
}
// ANCHOR_END: all
