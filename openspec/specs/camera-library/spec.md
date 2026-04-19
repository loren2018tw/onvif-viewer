# Capability: Camera Library

## Purpose

定義系統如何儲存、載入、更新與刪除已保存的攝影機資料，並維持前端可操作的攝影機清單。

## Requirements

### Requirement: 載入已儲存攝影機

系統 SHALL 從本機設定目錄載入已儲存的攝影機清單。

#### Scenario: 首次啟動尚未建立資料檔

- GIVEN 本機設定目錄中尚未存在 cameras.json
- WHEN 系統讀取已儲存攝影機
- THEN 系統回傳空清單

#### Scenario: 已存在資料檔

- GIVEN 本機設定目錄中存在 cameras.json
- WHEN 系統讀取已儲存攝影機
- THEN 系統解析 JSON 內容為攝影機清單
- AND 系統回傳完整的攝影機資訊與時間戳

### Requirement: 新增攝影機

系統 SHALL 支援建立新的攝影機紀錄並持久化到本機儲存。

#### Scenario: 儲存手動輸入的攝影機

- GIVEN 使用者在新增攝影機表單中填入名稱、位址、ONVIF 連接埠、帳號與密碼
- WHEN 使用者送出儲存
- THEN 系統建立新的攝影機識別碼
- AND 系統設定 created_at 與 updated_at 為目前時間
- AND 系統將新攝影機寫入本機 cameras.json

#### Scenario: 使用搜尋結果新增攝影機

- GIVEN 使用者從搜尋結果開啟新增攝影機表單
- WHEN 使用者確認並儲存攝影機
- THEN 系統保留搜尋結果預填的位址與連接埠
- AND 系統將使用者輸入的帳號密碼一併保存

### Requirement: 以位址與連接埠去除重複紀錄

系統 SHALL 以位址與 ONVIF 連接埠組合作為新增時的去重條件。

#### Scenario: 儲存已存在同位址同連接埠的攝影機

- GIVEN 本機清單中已存在相同位址與 ONVIF 連接埠的攝影機
- WHEN 使用者再次儲存相同位址與連接埠的攝影機
- THEN 系統移除舊紀錄後寫入新紀錄
- AND 系統最終只保留一筆相同位址與連接埠的資料

### Requirement: 編輯攝影機

系統 SHALL 支援更新既有攝影機的基本連線資訊。

#### Scenario: 更新攝影機名稱與認證資料

- GIVEN 使用者開啟已儲存攝影機的編輯表單
- WHEN 使用者修改名稱、位址、連接埠、帳號或密碼並儲存
- THEN 系統更新對應識別碼的攝影機紀錄
- AND 系統將 updated_at 更新為目前時間

#### Scenario: 編輯時保留既有補充欄位

- GIVEN 既有攝影機紀錄已有 created_at、stream_uri、manufacturer 或 model
- WHEN 使用者透過編輯表單更新基本連線資訊
- THEN 系統保留原有 created_at
- AND 系統在新輸入未提供時保留既有 stream_uri、manufacturer 與 model

### Requirement: 刪除攝影機

系統 SHALL 支援刪除已儲存的攝影機。

#### Scenario: 從已儲存清單刪除攝影機

- GIVEN 使用者在已儲存攝影機清單中選擇刪除某筆攝影機
- WHEN 系統完成刪除
- THEN 該攝影機不再出現在本機清單中

#### Scenario: 刪除目前選取的攝影機

- GIVEN 使用者刪除的攝影機正是目前選取中的攝影機
- WHEN 刪除完成
- THEN 前端清除目前選取的攝影機狀態

### Requirement: 記住最近一次輸入的帳號密碼

系統 SHALL 在新增攝影機對話框預填最近一次成功儲存時使用的帳號密碼。

#### Scenario: 連續新增多台攝影機

- GIVEN 使用者已經儲存過至少一台攝影機
- WHEN 使用者再次開啟新增攝影機對話框
- THEN 系統以前一次儲存時的帳號與密碼作為預設值
