#!/usr/bin/env bash
# 统一 Release 正文格式：
# - latest 使用固定预览模板
# - 正式版优先使用 docs/releases/<tag>.md
# - 没有单独文件时从 CHANGELOG.md 提取对应版本
set -euo pipefail

usage() {
  echo "usage: $0 <tag|latest> <output-file>" >&2
  exit 2
}

tag="${1:-}"
output="${2:-}"
if [ -z "$tag" ] || [ -z "$output" ]; then
  usage
fi

repo="${GITHUB_REPOSITORY:-rootphantomer/quake_rs}"
sha="${GITHUB_SHA:-$(git rev-parse HEAD)}"
release_notes_dir="docs/releases"

install_notes() {
  cat <<EOF

---

### 安装

从 [Releases](https://github.com/${repo}/releases) 页面下载对应平台的二进制：

| 平台 | 文件 |
|------|------|
| Linux x64 | \`quake-x86_64-unknown-linux-gnu.tar.gz\` |
| Linux ARM64 | \`quake-aarch64-unknown-linux-gnu.tar.gz\` |
| macOS Intel | \`quake-x86_64-apple-darwin.tar.gz\` |
| macOS Apple Silicon | \`quake-aarch64-apple-darwin.tar.gz\` |
| macOS 通用 | \`quake-universal-apple-darwin.tar.gz\` |
| Windows x64 | \`quake-x86_64-pc-windows-msvc.zip\` |

或使用 cargo：

\`\`\`bash
cargo install quake --git https://github.com/${repo}
\`\`\`
EOF
}

{
  if [ "$tag" = "latest" ]; then
    short_sha="${sha:0:7}"
    cat <<EOF
## Latest Build

\`master\` 分支最新构建，包含最新提交，不保证稳定。正式版本请使用 [最新 Release](https://github.com/${repo}/releases/latest)。

- 提交：[\`${short_sha}\`](https://github.com/${repo}/commit/${sha})
- 构建时间：$(date -u '+%Y-%m-%d %H:%M UTC')
- 完整变更：[master](https://github.com/${repo}/commits/master)
EOF
  else
    custom_notes="${release_notes_dir}/${tag}.md"
    if [ -f "$custom_notes" ]; then
      cat "$custom_notes"
      if [ -s "$custom_notes" ] && [ -n "$(tail -c 1 "$custom_notes")" ]; then
        echo
      fi
    else
      version="${tag#v}"
      changelog_body="$(
        awk -v version="$version" '
          index($0, " v" version ":") > 0 && /^- [0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9] / {
            found = 1
            next
          }
          found && /^- [0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9] / { exit }
          found && /^  - / {
            sub(/^  /, "")
            print
          }
        ' CHANGELOG.md
      )"

      if [ -z "$changelog_body" ]; then
        echo "CHANGELOG.md has no release section for ${tag}" >&2
        exit 1
      fi

      printf '## quake %s\n\n%s\n' "$tag" "$changelog_body"
    fi

    previous_tag="$(git describe --tags --abbrev=0 "${tag}^" 2>/dev/null || true)"
    echo
    if [ -n "$previous_tag" ]; then
      echo "**完整变更**：[${previous_tag}...${tag}](https://github.com/${repo}/compare/${previous_tag}...${tag})"
    else
      echo "**完整变更**：[查看全部提交](https://github.com/${repo}/commits/${tag})"
    fi
  fi

  install_notes
} > "$output"
