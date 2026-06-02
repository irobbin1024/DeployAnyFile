# DeployAnyFile

上传任意媒体文件并通过短链分享的轻量级 Web 应用。

- **后端**：Rust + Axum + SQLite（sqlx），JWT 鉴权，bcrypt 密码哈希
- **前端**：Vue 3 + Vite + vue-router
- **部署**：单容器 Docker（多阶段构建，运行镜像精简）

## 功能

- 首页正中上传区，点击或拖拽上传图片 / 音频 / 视频 / HTML / Markdown / 文本等
- 上传后自动生成短链（base62 随机、保证唯一），也可在上传时自定义链接地址
- 用户系统：注册、登录、修改密码；超级管理员 `admin`（密码预设、可改）可管理其他用户
- 文件列表：分类过滤、文件名搜索、分页、多选删除 / 取消（开启）分享
- 分享详情弹窗：分享创建时间、总浏览数、独立 IP 数、访问记录（IP / 时间 / 设备）
- 公开预览页 `/p/:slug`：按类型在线渲染；若访问者是文件所有者，额外显示管理面板（改链接、开关分享、查看数据、删除）

## 快速开始（Docker）

```bash
# 1. 修改 docker-compose.yml 里的 JWT_SECRET 和 ADMIN_PASSWORD
# 2. 构建并启动
docker compose up -d --build
```

打开 http://localhost:8080 ，用 `admin / admin123`（或你设置的密码）登录。

数据（SQLite 数据库 + 上传文件）持久化在名为 `app-data` 的 Docker volume 中。

## 配置（环境变量）

| 变量 | 默认值 | 说明 |
| --- | --- | --- |
| `BIND_ADDR` | `0.0.0.0:8080` | 监听地址 |
| `DATA_DIR` | `./data` | 数据库与上传文件根目录 |
| `JWT_SECRET` | `change-me...` | 登录令牌签名密钥，**务必修改** |
| `ADMIN_USERNAME` | `admin` | 首次启动时创建的超级管理员账号 |
| `ADMIN_PASSWORD` | `admin123` | 超级管理员初始密码（之后可在界面修改） |
| `MAX_UPLOAD_MB` | `100` | 单文件最大体积（MB） |
| `STATIC_DIR` | `./static` | 前端静态文件目录（Docker 内自动设置） |

> 修改管理员密码：登录后在首页右上「修改密码」。`ADMIN_PASSWORD` 仅在该账号**首次创建**时生效。

## 本地开发

需要 Rust 1.79+ 与 Node 20+。

**后端**（终端 1）：

```bash
cd backend
cargo run
# 监听 http://localhost:8080
```

**前端**（终端 2）：

```bash
cd frontend
npm install
npm run dev
# 打开 http://localhost:5173 ，已配置代理到后端
```

## 项目结构

```
DeployAnyFile/
├── backend/                 # Rust / Axum API + 静态文件服务
│   ├── src/
│   │   ├── main.rs          # 路由装配、启动
│   │   ├── config.rs        # 环境变量配置
│   │   ├── db.rs            # 连接池、迁移、管理员引导
│   │   ├── auth.rs          # JWT、密码哈希、鉴权提取器
│   │   ├── error.rs         # 统一错误 -> HTTP 响应
│   │   ├── models.rs        # 数据模型与请求体
│   │   ├── util.rs          # 短链生成、校验、分类
│   │   └── handlers/        # auth / users / files / public 接口
│   └── migrations/          # SQLite 建表脚本
├── frontend/                # Vue 3 单页应用
│   └── src/views/           # Login / Register / Home / Admin / Preview
├── Dockerfile               # 三阶段构建
├── docker-compose.yml
└── README.md
```

## API 概览

所有 `/api/*` 接口（除登录注册、公开预览外）需在请求头携带 `Authorization: Bearer <token>`。

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| POST | `/api/auth/register` | 注册 |
| POST | `/api/auth/login` | 登录，返回 token |
| GET | `/api/auth/me` | 当前用户 |
| POST | `/api/auth/change-password` | 修改自己密码 |
| GET | `/api/users` | 用户列表（管理员） |
| POST | `/api/users` | 新建用户（管理员） |
| DELETE | `/api/users/:id` | 删除用户（管理员） |
| POST | `/api/users/:id/reset-password` | 重置密码（管理员） |
| POST | `/api/files/upload` | 上传文件（multipart，可带 `slug`） |
| GET | `/api/files` | 文件列表（`category`/`search`/`page`/`page_size`） |
| DELETE | `/api/files` | 批量删除（body `{ids}`） |
| POST | `/api/files/share` | 批量开关分享（body `{ids,is_shared}`） |
| PATCH | `/api/files/:id/share` | 单个开关分享 |
| PATCH | `/api/files/:id/slug` | 修改链接地址 |
| GET | `/api/files/:id/stats` | 分享统计 |
| GET | `/api/public/:slug` | 公开元数据（记录访问） |
| GET | `/raw/:slug` | 原始文件内容 |

## 安全说明

- 上传的 HTML 在预览页通过带 `sandbox` 的 iframe 渲染，限制其能力。
- 取消分享后，非所有者无法访问预览页与原始文件。
- 公网部署务必修改 `JWT_SECRET` 与管理员密码，并建议置于 HTTPS 反向代理之后（已支持 `X-Forwarded-For` 识别真实 IP）。
