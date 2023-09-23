use std::{
    collections::{HashMap, HashSet},
    fs,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use queues::IsQueue;
use regex::Regex;

static KILL_SWITCH: AtomicBool = AtomicBool::new(false);

pub async fn search(root: String) {
    let kill_switch = Arc::new(AtomicBool::new(false));
    let kill_switch2 = kill_switch.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            kill_switch2.store(true, Ordering::Relaxed);
        }
    });

    let mut map = HashMap::new();

    let mut queue = queues::Queue::<String>::new();
    queue.add(root).unwrap();

    while let Ok(path) = queue.remove() {
        if kill_switch.load(Ordering::Relaxed) {
            println!("Saving {}", map.len());
            save_to_disk(&map);
            kill_switch.store(false, Ordering::SeqCst);
        }

        let url = format!("https://en.wikipedia.org/wiki/{path}");
        if map.contains_key(&path) {
            println!("DUP {url}");
            continue;
        }
        println!("Searching {path}");

        if KILL_SWITCH.load(Ordering::Relaxed) {}

        let resp = reqwest::get(url).await.unwrap();

        let html = resp.text().await.unwrap();
        let s1 = Regex::new("<a href=\"/wiki/(.*)</a>").unwrap();
        let responses = s1.captures_iter(&html);
        let mut references = HashSet::new();
        for cap in responses {
            let (_, [capture]) = cap.extract::<1>();

            let (href, _) = capture.split_once('"').unwrap();
            let href = href.split('#').next().unwrap();
            if href.contains(':') || href.contains("%") {
                continue;
            }
            references.insert(href.to_owned());
        }

        map.insert(path, references.clone().into_iter().collect());
        for sub in references {
            queue.add(sub).unwrap();
        }
    }
    println!("Finished. ");
    save_to_disk(&map)
}

fn save_to_disk(map: &HashMap<String, Vec<String>>) {
    let len = map.len();
    let bin = serde_json::to_string_pretty(&map).unwrap();
    fs::write(format!("map_of_{len}.json"), &bin).unwrap();
}
