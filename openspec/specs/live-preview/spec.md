# Capability: Live Preview

## Purpose

定義系統如何將攝影機 RTSP 串流轉成本機 MJPEG 預覽，並管理預覽生命週期與前端顯示狀態。

## Requirements

### Requirement: 啟動即時預覽

系統 SHALL 支援將攝影機 RTSP 串流轉為本機 MJPEG HTTP 預覽。

#### Scenario: 預覽已儲存攝影機

- GIVEN 使用者已選取一台已儲存攝影機
- WHEN 使用者在主畫面按下預覽
- THEN 系統先取得所選碼流類型的 RTSP URI
- AND 系統啟動 FFmpeg 將 RTSP 串流轉為 MJPEG
- AND 系統回傳本機 http://127.0.0.1 隨機連接埠的 /stream 預覽 URL

#### Scenario: 選擇主碼流或副碼流預覽

- GIVEN 使用者可在主畫面選擇主碼流或副碼流
- WHEN 使用者啟動預覽
- THEN 系統依照所選碼流類型取得對應的 RTSP URI

### Requirement: 啟動預覽前檢查 FFmpeg

系統 SHALL 在啟動即時預覽前確認執行環境中可用 FFmpeg。

#### Scenario: 系統已安裝 FFmpeg

- GIVEN 執行環境可成功執行 ffmpeg -version
- WHEN 使用者啟動預覽
- THEN 系統允許啟動 FFmpeg 預覽流程

#### Scenario: 系統未安裝 FFmpeg

- GIVEN 執行環境無法執行 ffmpeg -version
- WHEN 使用者啟動預覽或應用程式檢查 FFmpeg 狀態
- THEN 系統回報 FFmpeg 未安裝
- AND 系統提供依作業系統對應的安裝指令

### Requirement: 預覽流程一次只維持一個作用中的串流

系統 SHALL 在同一時間只維持一個作用中的即時預覽串流。

#### Scenario: 啟動新的預覽

- GIVEN 目前已有一個作用中的即時預覽
- WHEN 系統準備啟動新的預覽
- THEN 系統先停止現有預覽
- AND 系統再啟動新的 FFmpeg 與 MJPEG 服務

### Requirement: 提供可持續輸出的 MJPEG 端點

系統 SHALL 透過本機 HTTP 端點持續輸出 multipart MJPEG 影像幀。

#### Scenario: 預覽端點被讀取

- GIVEN FFmpeg 已開始輸出 JPEG 幀
- WHEN 用戶端請求本機 /stream 端點
- THEN 系統以 multipart/x-mixed-replace 回應 JPEG 幀資料
- AND 系統對每幀附帶正確的 Content-Type 與 Content-Length

#### Scenario: 讀取端落後於最新幀

- GIVEN 預覽幀以廣播佇列分送到端點消費者
- WHEN 某個消費者落後而發生 lagged 狀態
- THEN 系統略過落後幀並繼續傳送後續幀

### Requirement: 控制預覽生命週期與前端狀態

系統 SHALL 支援使用者停止預覽，並在串流異常時清理預覽狀態。

#### Scenario: 使用者手動停止預覽

- GIVEN 目前存在作用中的預覽
- WHEN 使用者按下停止
- THEN 系統關閉本機 MJPEG 服務
- AND 系統中止 JPEG 讀取工作
- AND 系統終止 FFmpeg 行程
- AND 前端清除預覽 URL 與預覽中狀態

#### Scenario: 瀏覽器中的 MJPEG 影像載入失敗

- GIVEN 預覽影像元素發生載入錯誤
- WHEN 前端收到影像錯誤事件
- THEN 前端呼叫停止預覽
- AND 系統清除預覽中狀態

### Requirement: 於預覽中顯示目前串流 URI

系統 SHALL 在預覽啟動後提供目前串流 URI 供使用者查看與複製。

#### Scenario: 預覽已啟動且串流 URI 可用

- GIVEN 系統已取得目前攝影機的串流 URI
- WHEN 預覽畫面顯示成功
- THEN 前端在預覽區顯示目前串流 URI
- AND 使用者可將該 URI 複製到剪貼簿

### Requirement: 自動預覽已選攝影機

系統 SHALL 支援在使用者切換攝影機選取時自動啟動預覽。

#### Scenario: 已啟用自動預覽

- GIVEN 使用者已開啟自動預覽選項
- WHEN 使用者從已儲存清單選擇某台攝影機
- THEN 系統自動取得串流 URI
- AND 系統自動啟動該攝影機的即時預覽

#### Scenario: 未啟用自動預覽

- GIVEN 使用者未開啟自動預覽選項
- WHEN 使用者切換目前選取的攝影機
- THEN 系統停止現有預覽
- AND 系統等待使用者手動重新啟動預覽
