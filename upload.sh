#!/usr/bin/env bash
#
# 命令行上传工具：把本地文件传到 DeployAnyFile 站点并打印分享链接。
#
# 用法：
#   ./upload.sh <文件路径> [自定义短链]
#
# 示例：
#   ./upload.sh report.html              # 随机短链
#   ./upload.sh notes.md my-notes        # 指定短链 /p/my-notes
#
# 配置（推荐用 API 令牌）：
#   在同目录放一个 upload.env 文件（已被 .gitignore 忽略）：
#        DAF_URL=https://your-domain
#        DAF_TOKEN=daf_xxxxxxxx...        # 在网页「API 令牌」页生成，推荐
#   或退而用账号密码（脚本会自动登录换取 token）：
#        DAF_USER=irobbin
#        DAF_PASS=你的密码
#   也可直接用同名环境变量。
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
[ -f "$SCRIPT_DIR/upload.env" ] && source "$SCRIPT_DIR/upload.env"

BASE_URL="${DAF_URL:-http://localhost:8080}"
# 分享链接的地址；开发模式下后端(8080)与前端(5173)分离时用它指向前端，
# 不设则沿用 BASE_URL（生产单端口部署即同源，无需设置）。
SHARE_URL="${DAF_SHARE_URL:-$BASE_URL}"
TOKEN="${DAF_TOKEN:-}"
USERNAME="${DAF_USER:-}"
PASSWORD="${DAF_PASS:-}"

FILE="${1:-}"
SLUG="${2:-}"

if [ -z "$FILE" ]; then
  echo "用法: $0 <文件路径> [自定义短链]" >&2
  exit 1
fi
if [ ! -f "$FILE" ]; then
  echo "✗ 文件不存在: $FILE" >&2
  exit 1
fi

# 1) 取得 token：优先用配置的 API 令牌，否则用账号密码登录换取
if [ -z "$TOKEN" ]; then
  if [ -z "$USERNAME" ] || [ -z "$PASSWORD" ]; then
    echo "✗ 未配置凭证。请在 upload.env 设置 DAF_TOKEN，或 DAF_USER/DAF_PASS" >&2
    exit 1
  fi
  TOKEN=$(curl -fsS --connect-timeout 15 -X POST "$BASE_URL/api/auth/login" \
    -H 'Content-Type: application/json' \
    -d "{\"username\":\"$USERNAME\",\"password\":\"$PASSWORD\"}" \
    | python3 -c 'import sys,json;print(json.load(sys.stdin)["token"])') \
    || { echo "✗ 登录失败，检查地址/账号/密码" >&2; exit 1; }
fi

# 2) 上传文件（可带自定义短链）
CURL_ARGS=(-F "file=@$FILE")
[ -n "$SLUG" ] && CURL_ARGS+=(-F "slug=$SLUG")

RESP=$(curl -fsS --connect-timeout 15 -X POST "$BASE_URL/api/files/upload" \
  -H "Authorization: Bearer $TOKEN" "${CURL_ARGS[@]}") \
  || { echo "✗ 上传失败：检查 DAF_URL 是否正确、服务是否在线，或短链是否已被占用" >&2; exit 1; }

# 3) 解析 slug 并打印分享链接
RSLUG=$(echo "$RESP" | python3 -c 'import sys,json;print(json.load(sys.stdin)["slug"])')
LINK="$SHARE_URL/p/$RSLUG"
echo "✓ 上传成功"
echo "分享链接: $LINK"

# macOS：顺便复制到剪贴板
command -v pbcopy >/dev/null && printf '%s' "$LINK" | pbcopy && echo "（已复制到剪贴板）"
