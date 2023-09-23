// Det er her eventuel kommunikation med bfs søge-maskinen er.

// Lige nu skal den bare kunne hente dataen

use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
    time::Duration,
};

use once_cell::sync::Lazy;
use queues::{IsQueue, Queue};
use regex::Regex;
use regex_macro::regex;
use tokio::sync::RwLock;

static DATA: Lazy<RwLock<HashMap<String, Box<[String]>>>> = Lazy::new(|| {
    tokio::spawn(data_loader());
    RwLock::new(HashMap::new())
});

pub async fn init() {
    drop(DATA.read().await);
    while DATA.read().await.len() == 0 {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn data_loader() {
    loop {
        let mut all_root_files = std::fs::read_dir("./").unwrap();
        let mut biggest_file = None;

        while let Some(Ok(entry)) = all_root_files.next() {
            let Ok(e_name) = entry.file_name().into_string() else {
                continue;
            };
            static RE: Lazy<&Regex> = Lazy::new(|| regex!("map_of_(.*).json"));

            let Some(cap) = RE.captures(&e_name) else {
                continue;
            };
            let (_, [n_str]) = cap.extract::<1>();

            let Ok(n) = n_str.parse::<usize>() else {
                continue;
            };
            if let Some((_, o_n)) = &biggest_file {
                if n > *o_n {
                    biggest_file = Some((e_name, n));
                }
            } else {
                biggest_file = Some((e_name, n));
            }
        }

        let Some((biggest_file, size)) = biggest_file else {
            eprintln!("FATAL. Failed to find dataset");
            std::process::exit(1);
        };

        let Ok(bin) = std::fs::read(&biggest_file) else {
            eprintln!("FATAL. Failed to read dataset");
            std::process::exit(2);
        };

        let Ok(data) = serde_json::from_slice(&bin) else {
            eprintln!("FATAL. Failed to parse dataset");
            std::process::exit(3);
        };
        let mut lock = DATA.write().await;
        if size > lock.len() {
            *lock = data;
            println!("Updating dataset to size {size}");
        }
        drop(lock);

        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}

pub async fn search(root: &str, target: &str) -> Option<(u32, Box<[String]>)> {
    if root == "Main_Page" {
        return None;
    }
    let mut walked = HashSet::new();
    let mut queue: Queue<(&str, u32)> = Queue::new();
    queue.add((root, 0)).unwrap();
    let map = DATA.read().await;

    while let Ok((next, depth)) = queue.remove() {
        if next == target {
            let subs = map
                .get(root)
                .unwrap()
                .iter()
                .map(ToOwned::to_owned)
                .collect();

            return Some((depth, subs));
        }

        let Some(subs) = map.get(next).map(|s| s.deref()) else {
            continue;
        };

        for sub in subs {
            if !walked.contains(sub) {
                walked.insert(sub);
                queue.add((sub.deref(), depth + 1)).ok().unwrap();
            }
        }
    }

    None
}

pub async fn get_target() -> String {
    let map = DATA.read().await;
    let mut keys = map.keys();
    let index = rand::random::<usize>() % keys.len();
    keys.nth(index).unwrap().to_owned()
}
