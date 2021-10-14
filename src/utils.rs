use std::{fmt, fs};

#[derive(Debug, Clone)]
pub struct FileRecoder {
    pub dirname: String,
    pub filename: Option<String>,
    pub extension: Option<String>,
}

impl FileRecoder {
    pub fn new(path: &str) -> Self {
        let (dirname, last) = split_last_pattern(path, '/');
        let last = last.unwrap();
        let mut filename = Some(last.clone());
        let mut extension = None;
        if last.contains(".") {
            let tmp = split_last_pattern(&last, '.');
            filename.replace(tmp.0);
            extension = tmp.1;
        }
        FileRecoder {
            dirname,
            filename,
            extension,
        }
    }

    pub fn join(&self, path: &str) -> Self {
        let mut result: Vec<_> = self.dirname.split('/').collect();
        for item in path.split('/') {
            if item != "." {
                if item == ".." && result.last().is_some() && result.last() != Some(&"..") {
                    result.pop();
                } else {
                    result.push(item);
                }
            }
        }
        Self::new(&result.join("/"))
    }

    pub fn complete_path(&mut self) -> bool {
        let extensions = ["js", "jsx", "ts", "tsx"];
        let check_exist = |path: String| fs::File::open(path).is_ok();
        if check_exist(self.to_string()) {
            return true;
        }
        self.filename.as_mut().unwrap().push_str(
            &self
                .extension
                .take()
                .map_or(String::new(), |x| format!(".{}", x)),
        );
        for ext in extensions.iter() {
            self.extension.replace(ext.to_string());
            if check_exist(self.to_string()) {
                return true;
            }
        }

        self.dirname
            .push_str(&format!("/{}", self.filename.take().unwrap()));
        self.filename.replace(String::from("index"));
        for ext in extensions.iter() {
            self.extension.replace(ext.to_string());
            if fs::File::open(self.to_string()).is_ok() {
                return true;
            }
        }

        false
    }

    pub fn read_line(&mut self) -> Vec<String> {
        fs::read_to_string(self.to_string())
            .expect("file doesn't exist")
            .split('\n')
            .map(|x| x.trim_start().to_string())
            .filter(|x| !(x.starts_with("//") || x.starts_with("/*") || x.ends_with("*/")))
            .collect::<Vec<_>>()
    }
}

impl fmt::Display for FileRecoder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}.{}",
            self.dirname,
            self.filename.as_ref().unwrap_or(&"".to_string()),
            self.extension.as_ref().unwrap_or(&"".to_string())
        )
    }
}

fn split_last_pattern(s: &str, pattern: char) -> (String, Option<String>) {
    let pos = s.chars().rev().position(|c| c == pattern);
    match pos {
        None => (s.to_string(), None),
        Some(idx) => {
            let pos = s.len() - idx - 1;
            (s[..pos].to_string(), Some(s[pos + 1..].to_string()))
        }
    }
}
