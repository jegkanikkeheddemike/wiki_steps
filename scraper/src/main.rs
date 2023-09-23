mod search;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    //Common_Language_Runtime
    search::search("Common_Language_Runtime".into()).await;
}
