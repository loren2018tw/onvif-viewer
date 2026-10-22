# Issue tracker：Local Markdown

這個 repo 的 issue 與 spec 以 markdown 檔案形式存放在 `.scratch/`。

## 慣例

- 每個 feature 一個目錄：`.scratch/<feature-slug>/`
- Spec 為 `.scratch/<feature-slug>/spec.md`
- 實作 issue 為一 ticket 一檔：`.scratch/<feature-slug>/issues/<NN>-<slug>.md`，從 `01` 開始編號，不可合併成單一 tickets 檔
- Triage 狀態記錄在每個 issue 檔案頂端的 `Status:` 行（角色字串見 `triage-labels.md`）
- 留言與對話歷史附加在檔案底部的 `## Comments` 標題下

## 當 skill 說「publish to the issue tracker」

在 `.scratch/<feature-slug>/` 下建立新檔案（必要時建立目錄）。

## 當 skill 說「fetch the relevant ticket」

讀取指定路徑的檔案。使用者通常會直接給路徑或 issue 編號。

## Wayfinding 操作

供 `/wayfinder` 使用。**map** 是一個檔案，每個 ticket 一個 **child** 檔案。

- **Map**：`.scratch/<effort>/map.md`（Notes / Decisions-so-far / Fog 內容）。
- **Child ticket**：`.scratch/<effort>/issues/NN-<slug>.md`，從 `01` 編號，body 放問題。`Type:` 行記錄 ticket 類型（`research`/`prototype`/`grilling`/`task`）；`Status:` 行記錄 `claimed`/`resolved`。
- **Blocking**：頂端附近的 `Blocked by: NN, NN` 行。當列出的每個檔案都是 `resolved`，ticket 即解除封鎖。
- **Frontier**：掃描 `.scratch/<effort>/issues/`，找出 open、未封鎖、未 claim 的檔案；編號最小者優先。
- **Claim**：在任何工作開始前設定 `Status: claimed` 並存檔。
- **Resolve**：在 `## Answer` 標題下附加答案，設定 `Status: resolved`，再將 context pointer（gist + link）附加到 `map.md` 的 Decisions-so-far。
