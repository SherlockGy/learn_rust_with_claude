use crate::counter::CountResult;

/// 格式化输出统计结果
pub fn print_result(result: &CountResult, filename: Option<&str>) {
    match filename {
        Some(name) => {
            println!(
                "{:>8}{:>8}{:>8} {}",
                result.lines, result.words, result.chars, name
            );
        }
        None => {
            println!(
                "{:>8}{:>8}{:>8}",
                result.lines, result.words, result.chars
            );
        }
    }
}
