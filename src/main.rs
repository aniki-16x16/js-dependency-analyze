pub mod utils;

use std::{
    collections::HashMap,
    fs,
    io::{Result, Write},
    time::Instant,
};

use clap::{App, Arg};
use indicatif::ProgressBar;
use itertools::Itertools;
use regex::Regex;
use serde::Serialize;
use utils::FileRecoder;

fn main() -> Result<()> {
    let matches = App::new("JS Dependency Analyzer")
        .version("0.1.0")
        .arg(
            Arg::with_name("path")
                .help("root file")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("alias")
                .help("alias path")
                .short("a")
                .long("alias")
                .takes_value(true),
        )
        .get_matches();

    let re = Regex::new(
        r#"(?:^import.*from\s+['"](.*)['"]|require\(['"](.*)['"]\)|\s+from\s+['"](.*)['"])"#,
    )
    .unwrap();
    let pb = ProgressBar::new(1);
    let timer = Instant::now();

    let root = FileRecoder::new(matches.value_of("path").unwrap());
    let mut paths = vec![root.clone()];
    let alias: HashMap<&str, &str> = matches
        .value_of("alias")
        .map_or(vec![], |x| x.split(',').collect())
        .iter()
        .map(|x| {
            let mut tmp = x.split('=');
            (tmp.next().unwrap(), tmp.next().unwrap())
        })
        .collect();
    let mut adj_matirx: HashMap<String, Vec<usize>> =
        vec![(root.to_string(), vec![])].into_iter().collect();
    let mut path_idx_map: HashMap<String, usize> =
        vec![(root.to_string(), 0)].into_iter().collect();

    while paths.len() > 0 {
        pb.inc(1);
        let mut current = paths.pop().unwrap();
        for line in &current.read_import_line() {
            let caps = re.captures(line);
            if caps.is_some() {
                let caps = caps.unwrap();
                let mut name = caps
                    .get(1)
                    .or(caps.get(2).or(caps.get(3)))
                    .unwrap()
                    .as_str()
                    .to_string();

                let mut is_relative_path = None;
                if name.starts_with("./") || name.starts_with("../") {
                    is_relative_path.replace(true);
                } else {
                    for (k, v) in &alias {
                        if name.starts_with(k) {
                            is_relative_path.replace(false);
                            name = name.replacen(k, v, 1);
                            break;
                        }
                    }
                }
                // None值表示是模块，不加入paths中
                if is_relative_path.is_some() {
                    // 使用了路径别名时与根路径进行拼接
                    let mut next_path = match is_relative_path.unwrap() {
                        true => current.join(&name),
                        false => root.join(&name),
                    };
                    if next_path.complete_path() {
                        // 保存补全后的真实有效路径，且未访问过的情况下才加入paths
                        name = next_path.to_string();
                        if adj_matirx.get(&name).is_none() {
                            paths.push(next_path);
                            pb.inc_length(1);
                            adj_matirx.insert(name.clone(), vec![]);
                            path_idx_map.insert(name.clone(), path_idx_map.len());
                        }
                        adj_matirx
                            .get_mut(&current.to_string())
                            .unwrap()
                            .push(path_idx_map[&name]);
                    }
                }
            }
        }
    }
    pb.finish();
    println!("expend {}ms\nserializing...", timer.elapsed().as_millis());

    // 搜索最小公共前缀
    let mut prefix_pos = root.dirname.len();
    for key in path_idx_map.keys() {
        for i in 0..prefix_pos {
            if &root.dirname[i..i + 1] != &key[i..i + 1] {
                prefix_pos = i - 1;
                break;
            }
        }
    }
    let total = path_idx_map.len();
    fs::File::create("result.json")?;
    let mut file = fs::OpenOptions::new().append(true).open("result.json")?;
    file.write(&[b'['])?;
    file.flush()?;
    // 按照升序排列键值对
    for (k, i) in path_idx_map
        .drain()
        .map(|(k, i)| (k, i))
        .sorted_by(|a, b| a.1.cmp(&b.1))
    {
        let mut key = k.clone();
        key.replace_range(..prefix_pos, "$root/");
        let mut buf = serde_json::to_vec(&ResultAdjMaritx {
            name: key,
            verts: adj_matirx.remove(&k).unwrap(),
        })?;
        if i < total - 1 {
            buf.push(b',');
        }
        buf.push(b'\n');
        file.write(&buf)?;
        file.flush()?;
    }
    file.write(&[b']'])?;
    file.flush()?;
    Ok(())
}

#[derive(Debug, Serialize)]
struct ResultAdjMaritx {
    name: String,
    verts: Vec<usize>,
}
