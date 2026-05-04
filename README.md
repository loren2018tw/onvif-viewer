# ONVIF Viewer

一款基於 [Tauri](https://tauri.app/) 的跨平台桌面應用程式，用於探索、管理並即時預覽 ONVIF 相容 IP 攝影機串流。

## 功能簡介

- **攝影機自動探索**：透過 WS-Discovery 協議自動掃描區域網路上的 ONVIF 裝置，或手動輸入 IP 範圍進行掃描。
- **攝影機管理**：儲存多台攝影機的連線資訊（位址、ONVIF 連接埠、帳號密碼），並持久化於本地。
- **即時預覽**：取得 RTSP 串流 URI 並透過 HLS 代理轉換後在應用內播放，支援主、子串流切換。
- **連線診斷**：一鍵測試攝影機連線狀態，並取得詳細診斷報告以排除故障。
- **音訊支援**：預覽串流時可選擇是否啟用音訊。

## 技術架構

| 層級     | 技術                                  |
| -------- | ------------------------------------- |
| 桌面框架 | [Tauri 2](https://tauri.app/)         |
| 前端     | Vue 3 + TypeScript + Vite + Vuetify 3 |
| 後端     | Rust（tokio、axum、oxvif）            |
| 狀態管理 | Pinia                                 |
| 串流播放 | hls.js                                |

## 前置需求

- [Node.js](https://nodejs.org/) >= 18
- [pnpm](https://pnpm.io/) >= 9
- [Rust](https://www.rust-lang.org/tools/install)（穩定版工具鏈）
- Tauri 系統相依套件（Linux 需要 WebKit2GTK）

### Linux 系統相依套件（以 Debian / Ubuntu 為例）

```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev \
  libxdo-dev
```

## 建置與執行

### 安裝相依套件

```bash
pnpm install
```

### 開發模式（熱重載）

```bash
pnpm tauri dev
```

### 正式建置

```bash
pnpm tauri build
```

建置產物位於 `src-tauri/target/release/`，安裝包（`.deb` / NSIS）位於 `src-tauri/target/release/bundle/`。

## 專案結構

```
onvif-viewer/
├── frontend/          # Vue 3 前端原始碼
│   └── src/
│       ├── views/     # 頁面元件
│       ├── stores/    # Pinia 狀態管理
│       ├── types/     # TypeScript 型別定義
│       └── router/    # 路由設定
└── src-tauri/         # Rust 後端原始碼
    └── src/
        ├── camera.rs      # ONVIF 攝影機操作
        ├── commands.rs    # Tauri 指令（前後端橋接）
        ├── discovery.rs   # 攝影機探索邏輯
        ├── models.rs      # 資料模型
        ├── storage.rs     # 本地資料儲存
        └── stream.rs      # HLS 串流代理管理
```
