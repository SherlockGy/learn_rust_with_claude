/// 统计结果
pub struct CountResult {
    pub lines: usize,
    pub words: usize,
    pub chars: usize,
}

/// 统计文本的行数、单词数、字符数
pub fn count_text(text: &str) -> CountResult {
    let lines = text.lines().count();
    let words = text.split_whitespace().count();
    let chars = text.chars().count();

    CountResult { lines, words, chars }
}
