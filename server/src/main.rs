use std::io::stdin;

mod data;
#[tokio::main]
async fn main() {
    data::init().await;
    let target = data::get_target().await;

    for line in stdin().lines().map(Result::unwrap) {
        let Some((dist, subs)) = data::search(&line, &target).await else {
            println!("No path from {line} to target");
            continue;
        };
        println!("from {line} to target is {dist}");
        println!("{subs:#?}");
    }
}
