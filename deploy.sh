#!/usr/bin/env bash
#
# 一键部署到群晖 (Synology) NAS。
#   1. 通过 rsync 把本地代码同步到 NAS 上的项目目录（自动跳过 data / node_modules / target 等）
#   2. SSH 到 NAS，在该目录执行 docker compose up -d --build，重建镜像并重启容器
#
# 用法：
#   1. 复制 deploy.env.example 为 deploy.env，填入你的 NAS 信息
#   2. chmod +x deploy.sh
#   3. ./deploy.sh
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# ---- 默认配置（建议在 deploy.env 里覆盖，不要直接改这里）----
NAS_USER="${NAS_USER:-irobbin}"
NAS_HOST="${NAS_HOST:-nas.irobbin.com}"
NAS_PORT="${NAS_PORT:-1024}"
NAS_PATH="${NAS_PATH:-/volume1/docker/DeployAnyFile_SRC}"
# Container Manager 里的项目(project)名，用于让 CLI 更新同一套堆栈
NAS_PROJECT="${NAS_PROJECT:-deploy_any_file}"
# ------------------------------------------------------------

if [ -f "$SCRIPT_DIR/deploy.env" ]; then
  # shellcheck disable=SC1091
  source "$SCRIPT_DIR/deploy.env"
fi

REMOTE="$NAS_USER@$NAS_HOST"

echo "▶ 同步代码到 $REMOTE:$NAS_PATH ..."
rsync -az --delete \
  -e "ssh -p $NAS_PORT" \
  --exclude '.git' \
  --exclude 'node_modules' \
  --exclude 'target' \
  --exclude 'frontend/dist' \
  --exclude 'data' \
  --exclude 'deploy.env' \
  --exclude 'docker-compose.yml' \
  --exclude '.DS_Store' \
  "$SCRIPT_DIR/" "$REMOTE:$NAS_PATH/"

echo "▶ 在 NAS 上重建并重启容器（可能需要输入 sudo 密码）..."
ssh -t -p "$NAS_PORT" "$REMOTE" \
  "cd '$NAS_PATH' && sudo docker compose -p '$NAS_PROJECT' up -d --build && echo '--- 容器状态 ---' && sudo docker compose -p '$NAS_PROJECT' ps"

echo "✅ 部署完成，访问 http://$NAS_HOST:8080"
