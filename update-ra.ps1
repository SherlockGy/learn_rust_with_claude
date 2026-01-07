$settingsPath = ".vscode/settings.json"
$excludeDirs = @("target", ".git", ".idea", ".vscode", ".claude", "node_modules")

# 构建排除正则
$excludePattern = ($excludeDirs | ForEach-Object { [regex]::Escape($_) }) -join "|"
$excludeRegex = "[\\/]($excludePattern)[\\/]"

# 找到所有 Cargo.toml，排除指定目录
$projects = @()
Get-ChildItem -Recurse -Filter "Cargo.toml" -ErrorAction SilentlyContinue | ForEach-Object {
    if ($_.FullName -notmatch $excludeRegex) {
        $relativePath = $_.FullName.Replace("$PWD\", "").Replace("\", "/")
        $projects += $relativePath
    }
}

# 确保 .vscode 目录存在
if (!(Test-Path .vscode)) {
    New-Item -ItemType Directory -Path .vscode | Out-Null
}

# 读取现有配置
$otherSettings = @{}
if (Test-Path $settingsPath) {
    try {
        $content = Get-Content $settingsPath -Raw -Encoding UTF8
        if ($content) {
            $settings = $content | ConvertFrom-Json -AsHashtable
            foreach ($key in $settings.Keys) {
                if ($key -ne "rust-analyzer.linkedProjects") {
                    $otherSettings[$key] = $settings[$key]
                }
            }
        }
    } catch {}
}

# 手动构建格式化的 JSON
$lines = @()
$lines += "{"

# 先写入 linkedProjects
$lines += "    `"rust-analyzer.linkedProjects`": ["
for ($i = 0; $i -lt $projects.Count; $i++) {
    $comma = if ($i -lt $projects.Count - 1) { "," } else { "" }
    $lines += "        `"$($projects[$i])`"$comma"
}
if ($otherSettings.Count -gt 0) {
    $lines += "    ],"
} else {
    $lines += "    ]"
}

# 写入其他设置
$keys = @($otherSettings.Keys)
for ($i = 0; $i -lt $keys.Count; $i++) {
    $key = $keys[$i]
    $value = $otherSettings[$key]
    $jsonValue = ($value | ConvertTo-Json -Depth 10 -Compress)
    $comma = if ($i -lt $keys.Count - 1) { "," } else { "" }
    $lines += "    `"$key`": $jsonValue$comma"
}

$lines += "}"

# 写入文件
$lines -join "`r`n" | Set-Content $settingsPath -Encoding UTF8

Write-Host "Done! Found $($projects.Count) Rust projects:"
$projects | ForEach-Object { Write-Host "  - $_" }