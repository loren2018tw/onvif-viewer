# Capability: Camera Diagnostics

## Purpose

定義系統如何產生可供除錯的 ONVIF 診斷報告，並在介面中呈現與複製診斷結果。

## Requirements

### Requirement: 產生 ONVIF 診斷報告

系統 SHALL 支援對指定攝影機產生文字格式的 ONVIF 診斷報告。

#### Scenario: 成功建立 ONVIF 會話

- GIVEN 使用者在新增或編輯攝影機對話框中提供位址、連接埠、帳號與密碼
- WHEN 使用者觸發診斷
- THEN 系統建立 ONVIF 會話
- AND 診斷報告包含裝置位址與連接資訊
- AND 診斷報告標示 ONVIF 連線成功

#### Scenario: 無法建立 ONVIF 會話

- GIVEN 裝置不可達或提供的連線資訊無效
- WHEN 使用者觸發診斷
- THEN 系統回傳診斷報告
- AND 診斷報告明確標示無法建立連線
- AND 系統不再執行後續診斷步驟

### Requirement: 診斷報告包含服務端點與裝置資訊

系統 SHALL 在診斷報告中列出 ONVIF 服務能力與可讀取的裝置資訊。

#### Scenario: 可取得服務端點與裝置資訊

- GIVEN ONVIF 會話建立成功且帳號密碼可通過裝置資訊驗證
- WHEN 系統產生診斷報告
- THEN 報告列出 Device、Media、Media2 與 PTZ 端點 URL
- AND 報告列出製造商、型號、韌體版本與序號
- AND 報告標示帳號密碼驗證通過

#### Scenario: 無法取得裝置資訊

- GIVEN ONVIF 會話建立成功但裝置資訊請求失敗
- WHEN 系統產生診斷報告
- THEN 報告明確標示裝置資訊取得失敗
- AND 報告指出帳號密碼可能有誤

### Requirement: 診斷報告包含 profile 與串流 URI 測試結果

系統 SHALL 在診斷報告中列出媒體 profile 與各種串流 URI 取得結果。

#### Scenario: 裝置可回傳媒體 profiles

- GIVEN 裝置可回傳至少一個媒體 profile
- WHEN 系統產生診斷報告
- THEN 報告列出每個 profile 的名稱、token 與 video encoder token
- AND 報告分別標示 Media1 與 Media2 GetStreamUri 的成功、警告或失敗結果

#### Scenario: 裝置無法回傳媒體 profiles

- GIVEN 裝置的 get_profiles 請求失敗
- WHEN 系統產生診斷報告
- THEN 報告明確標示取得 profile 失敗

### Requirement: 診斷報告保留原始 SOAP 內容供除錯

系統 SHALL 在可行時附上第一個 profile 的原始 SOAP GetStreamUri 回應內容。

#### Scenario: 具備 Media 端點與至少一個 profile

- GIVEN 診斷流程已成功取得第一個媒體 profile 與 Media 端點 URL
- WHEN 系統對第一個 profile 發送原始 SOAP GetStreamUri 請求
- THEN 診斷報告附上 HTTP 狀態與原始 SOAP XML 內容

#### Scenario: 原始 SOAP 請求失敗

- GIVEN 診斷流程無法取得第一個 profile 的原始 SOAP 回應
- WHEN 系統產生診斷報告
- THEN 報告明確標示原始請求失敗

### Requirement: 前端呈現並複製診斷報告

系統 SHALL 在介面中顯示診斷報告，並允許使用者複製結果。

#### Scenario: 顯示診斷結果對話框

- GIVEN 診斷流程已完成
- WHEN 前端收到診斷報告文字
- THEN 前端開啟診斷報告對話框
- AND 將報告以等寬字體保留換行格式顯示

#### Scenario: 複製診斷報告

- GIVEN 診斷報告已顯示於對話框
- WHEN 使用者按下複製
- THEN 系統將完整診斷報告寫入剪貼簿
- AND 前端顯示複製成功提示
