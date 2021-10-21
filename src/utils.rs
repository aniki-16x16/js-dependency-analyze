use std::{fmt, fs};

#[derive(Debug, Clone)]
pub struct FileRecoder {
    pub dirname: String,
    pub filename: String,
    pub extension: Option<String>,
}

impl FileRecoder {
    pub fn new(path: &str) -> Self {
        let (dirname, last) = rsplit_once(path, '/');
        let last = last.unwrap();
        let mut filename = last.clone();
        let mut extension = None;
        if last.contains(".") {
            let tmp = rsplit_once(&last, '.');
            filename = tmp.0;
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
        let extensions = ["js", "jsx", "ts", "tsx"]; // 四种可省的扩展名
        let check_exist = |path: String| fs::File::open(path).is_ok();
        // 当前路径已经是完整路径
        if check_exist(self.to_string()) {
            return true;
        }
        // 否则extension应是文件名的一部分，将其拼接到filename的尾部
        self.filename.push_str(
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

        // 此时传入的路径应该是文件夹，在内部应有一个index文件作为入口(CommonJS规范)
        self.dirname.push_str(&format!("/{}", self.filename));
        self.filename = String::from("index");
        for ext in extensions.iter() {
            self.extension.replace(ext.to_string());
            if fs::File::open(self.to_string()).is_ok() {
                return true;
            }
        }

        false
    }

    pub fn read_import_line(&mut self) -> Vec<String> {
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
            self.filename,
            self.extension.as_ref().unwrap_or(&"".to_string())
        )
    }
}

fn rsplit_once(s: &str, pattern: char) -> (String, Option<String>) {
    let pos = s.chars().rev().position(|c| c == pattern);
    match pos {
        None => (s.to_string(), None),
        Some(idx) => {
            let pos = s.len() - idx - 1;
            (s[..pos].to_string(), Some(s[pos + 1..].to_string()))
        }
    }
}
