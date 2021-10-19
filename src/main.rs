pub mod utils;

use std::{collections::HashMap, time::Instant};

use clap::{App, Arg};
use indicatif::ProgressBar;
use regex::Regex;
use utils::FileRecoder;

fn main() {
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

    let re =
        Regex::new(r#"(?:import.*['"](.*)['"]|require\(['"](.*)['"]\)|\s+from\s+['"](.*)['"])"#)
            .unwrap();
    let pb = ProgressBar::new(1);
    let timer = Instant::now();
    let root = FileRecoder::new(matches.value_of("path").unwrap());
    let mut paths = vec![root.clone()];
    let mut alias = HashMap::new();
    for item in matches
        .value_of("alias")
        .map_or(vec![], |x| x.split(',').collect())
    {
        let pair = item.split('=').collect::<Vec<_>>();
        alias.insert(pair[0], pair[1]);
    }
    let mut ref_count = HashMap::new();

    while paths.len() > 0 {
        pb.inc(1);
        let mut current = paths.pop().unwrap();
        for line in &current.read_as_line() {
            let caps = re.captures(line);
            if caps.is_some() {
                let caps = caps.unwrap();
                let mut name = if caps.get(1).is_some() {
                    caps.get(1).unwrap()
                } else if caps.get(2).is_some() {
                    caps.get(2).unwrap()
                } else {
                    caps.get(3).unwrap()
                }
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
                        // 保存补全后的真实有效路径，未访问过的情况下才加入paths
                        name = next_path.to_string();
                        if ref_count.get(&name).is_none() {
                            paths.push(next_path);
                            pb.inc_length(1);
                        }
                    }
                }
                *ref_count.entry(name).or_insert(0) += 1;
            }
        }
    }
    pb.finish();

    // 搜索最小公共前缀并替换
    let mut prefix_pos = root.dirname.len();
    for (k, _) in &ref_count {
        if k.starts_with("/") {
            for i in 0..prefix_pos {
                if &root.dirname[i..i + 1] != &k[i..i + 1] {
                    prefix_pos = i - 1;
                    break;
                }
            }
        }
    }
    let mut result = HashMap::new();
    for (k, v) in ref_count.drain() {
        let mut key = k;
        if key.starts_with("/") {
            key.replace_range(..prefix_pos, "$root/");
        }
        result.insert(key, v);
    }
    println!("{:#?}\nexpend {}ms", result, timer.elapsed().as_millis());
}
