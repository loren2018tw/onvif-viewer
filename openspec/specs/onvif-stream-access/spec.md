# Capability: ONVIF Stream Access

## Purpose

定義系統如何驗證 ONVIF 攝影機連線，並以相容不同裝置的方式取得可用的 RTSP 串流 URI。

## Requirements

### Requirement: 驗證 ONVIF 裝置連線

系統 SHALL 支援使用 ONVIF Device Service 驗證攝影機連線與帳號密碼。

#### Scenario: 帳號密碼正確

- GIVEN 使用者提供可連線的 ONVIF 位址、連接埠、帳號與密碼
- WHEN 系統建立 ONVIF 會話並請求裝置資訊
- THEN 系統回報連線成功

#### Scenario: 帳號密碼錯誤或裝置不可達

- GIVEN 使用者提供無效認證資訊或不可達的 ONVIF 裝置
- WHEN 系統建立 ONVIF 會話或請求裝置資訊失敗
- THEN 系統回傳對應的失敗訊息

### Requirement: 取得 RTSP 串流 URI

系統 SHALL 能夠從 ONVIF 攝影機取得可用的 RTSP 或 RTSPS 串流 URI。

#### Scenario: 存在媒體 profile 且可直接取得有效 URI

- GIVEN 裝置可回傳至少一個媒體 profile
- WHEN 系統依優先順序向 Media1 或 Media2 請求 profile 的串流 URI
- THEN 系統回傳第一個有效的 RTSP 或 RTSPS URI

#### Scenario: 裝置沒有任何媒體 profile

- GIVEN 裝置沒有可用的媒體 profile
- WHEN 系統嘗試取得串流 URI
- THEN 系統回傳沒有可用媒體 profile 的錯誤

### Requirement: 依碼流類型優先挑選 profile

系統 SHALL 根據使用者要求的主碼流或副碼流，優先挑選名稱或 token 最相符的媒體 profile。

#### Scenario: 要求副碼流

- GIVEN 使用者要求副碼流
- WHEN 系統為可用 profiles 排序
- THEN 系統優先嘗試名稱或 token 含有 sub、secondary、extra、low、minor、2nd、small 或 sd 的 profile
- AND 系統降低主碼流類型 profile 的優先度

#### Scenario: 要求主碼流

- GIVEN 使用者要求主碼流
- WHEN 系統為可用 profiles 排序
- THEN 系統優先嘗試名稱或 token 含有 main、primary、high、major、master 或 hd 的 profile
- AND 系統降低副碼流類型 profile 的優先度

### Requirement: 支援 Media1、Media2 與 SOAP 回退

系統 SHALL 依序嘗試多種方式取得串流 URI，以提升不同 ONVIF 裝置的相容性。

#### Scenario: Media1 成功取得 URI

- GIVEN 某個 profile 的 Media1 GetStreamUri 回傳有效的 RTSP URI
- WHEN 系統嘗試取得串流 URI
- THEN 系統直接使用該 URI

#### Scenario: Media1 或 Media2 回傳無效 URI

- GIVEN profile 的 Media1 或 Media2 回傳內容不是有效的 RTSP 或 RTSPS URI
- WHEN 系統仍需取得可用串流
- THEN 系統保留最後一次裝置回傳內容作為錯誤上下文
- AND 系統繼續嘗試下一種取得方式

#### Scenario: XML 解析器無法正確處理 URI 中的實體編碼

- GIVEN 裝置實際回應的串流 URI 含有 XML 實體編碼例如 &amp;
- WHEN 標準 Media1 或 Media2 呼叫未能取得有效 RTSP URI
- THEN 系統改用原始 SOAP GetStreamUri 請求
- AND 系統從 SOAP XML 內的 Uri 節點還原出有效串流 URI

### Requirement: 在串流 URI 內嵌入認證資訊

系統 SHALL 在需要時將使用者名稱與密碼嵌入回傳的 RTSP URI。

#### Scenario: URI 尚未包含認證資訊

- GIVEN 系統已取得有效的 RTSP URI 且使用者名稱不是空值
- WHEN 系統準備回傳該 URI
- THEN 系統將經 URL 編碼的使用者名稱與密碼插入 URI authority 區段

#### Scenario: URI 已經包含認證資訊

- GIVEN 系統取得的 RTSP URI 在 authority 區段已包含認證資訊
- WHEN 系統準備回傳該 URI
- THEN 系統保留原始 URI 而不重複嵌入認證資訊

### Requirement: 無法取得任何有效串流 URI 時回報明確錯誤

系統 SHALL 在所有 profile 與回退方式皆失敗時回傳包含裝置最後回應內容的錯誤。

#### Scenario: 所有 profile 都無法產生有效 RTSP URI

- GIVEN 系統已嘗試所有排序後的 profiles、Media1、Media2 與原始 SOAP 回退
- WHEN 仍無法取得有效 RTSP URI
- THEN 系統回傳所有 profile 皆無法取得有效 RTSP URI 的錯誤
- AND 錯誤內容包含裝置最後回傳的原始 URI 或回應片段
