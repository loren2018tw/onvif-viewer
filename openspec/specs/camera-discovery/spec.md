# Capability: Camera Discovery

## Purpose

定義系統如何搜尋本地網路中的 ONVIF 攝影機，並將搜尋結果提供給後續加入攝影機流程使用。

## Requirements

### Requirement: 本地 ONVIF 自動搜尋

系統 SHALL 支援使用 WS-Discovery 在本地網路搜尋 ONVIF 攝影機。

#### Scenario: 使用預設搜尋時間啟動自動搜尋

- GIVEN 使用者位於主畫面的自動搜尋分頁
- WHEN 使用者觸發本地攝影機搜尋且未覆寫搜尋秒數
- THEN 系統使用 5 秒作為搜尋時間
- AND 系統回傳所有成功回應探測的裝置清單

#### Scenario: 從裝置資訊建立搜尋結果

- GIVEN 探測到的裝置提供至少一個 xaddr
- WHEN 系統解析第一個 xaddr
- THEN 系統從 URL 取得主機位址與連接埠
- AND 系統將裝置 scopes 中帶有 /name/ 的名稱解碼後填入搜尋結果
- AND 系統保留裝置回傳的所有 xaddrs

### Requirement: IP 範圍手動掃描

系統 SHALL 支援使用者輸入起始與結束 IPv4 位址，對指定 ONVIF 連接埠進行手動掃描。

#### Scenario: 掃描有效的 IP 範圍

- GIVEN 使用者提供有效的起始 IP、結束 IP 與連接埠
- WHEN 使用者啟動範圍掃描
- THEN 系統對區間內每個 IP 發送 ONVIF GetSystemDateAndTime SOAP 請求
- AND 系統僅回傳 HTTP 成功回應的裝置

#### Scenario: 起始 IP 大於結束 IP

- GIVEN 使用者提供的起始 IP 大於結束 IP
- WHEN 系統驗證掃描參數
- THEN 系統拒絕掃描請求
- AND 系統回傳起始 IP 必須小於或等於結束 IP 的錯誤

#### Scenario: 單一位址未回應 ONVIF 服務

- GIVEN 掃描中的某個 IP 或連接埠沒有可用的 ONVIF 服務
- WHEN 該請求逾時或回傳非成功狀態
- THEN 系統忽略該位址
- AND 系統繼續處理其餘位址

### Requirement: 搜尋結果供後續加入攝影機

系統 SHALL 在搜尋結果中保留足夠資訊，供使用者直接建立攝影機紀錄。

#### Scenario: 從搜尋結果開啟新增表單

- GIVEN 搜尋結果包含裝置位址、連接埠與可選名稱
- WHEN 使用者選擇將該結果加入攝影機清單
- THEN 系統以搜尋結果預填名稱、位址與 ONVIF 連接埠
- AND 系統允許使用者補上帳號密碼後儲存

### Requirement: 搜尋流程顯示執行中狀態

系統 SHALL 在自動搜尋與手動掃描進行期間提供載入中狀態。

#### Scenario: 自動搜尋進行中

- GIVEN 系統正在執行 WS-Discovery 搜尋
- WHEN 使用者仍停留在搜尋畫面
- THEN 系統顯示自動搜尋執行中的載入狀態

#### Scenario: 範圍掃描進行中

- GIVEN 系統正在執行 IP 範圍掃描
- WHEN 使用者仍停留在搜尋畫面
- THEN 系統顯示手動掃描執行中的載入狀態
