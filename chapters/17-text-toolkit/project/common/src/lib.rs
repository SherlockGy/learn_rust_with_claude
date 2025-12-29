// text-toolkit 共享库
// 提供文件操作的通用工具函数

use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

/// 文件统计信息
#[derive(Debug, Default)]
pub struct FileStats {
    /// 总行数
    pub lines: usize,
    /// 空行数
    pub blank: usize,
    /// 代码行数（非空行）
    pub code: usize,
    /// 字节数
    pub bytes: usize,
}

/// 统计单个文件
pub fn stats_file(path: &Path) -> io::Result<FileStats> {
    let file = fs::File::open(path)?;
    let metadata = file.metadata()?;
    let reader = BufReader::new(file);

    let mut stats = FileStats {
        bytes: metadata.len() as usize,
        ..Default::default()
    };

    for line in reader.lines() {
        let line = line?;
        stats.lines += 1;
        if line.trim().is_empty() {
            stats.blank += 1;
        } else {
            stats.code += 1;
        }
    }

    Ok(stats)
}

/// 安全写入文件（先写临时文件，再原子重命名）
pub fn safe_write(path: &Path, content: &str) -> io::Result<()> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, content)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

/// 确认提示
pub fn confirm(prompt: &str) -> bool {
    use std::io::Write;

    print!("{} (y/N) ", prompt);
    io::stdout().flush().ok();

    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();

    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_stats_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "line 1").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "line 3").unwrap();

        let stats = stats_file(file.path()).unwrap();
        assert_eq!(stats.lines, 3);
        assert_eq!(stats.blank, 1);
        assert_eq!(stats.code, 2);
    }
}
