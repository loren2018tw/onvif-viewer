# Domain Docs

工程相關 skill 在探索這個 repo 時，應如何讀取領域文件。

## 探索前先讀這些

- 根目錄的 **`GLOSSARY.md`**（single-context 佈局）
- **`docs/adr/`**：讀取與即將處理區域相關的 ADR

若上述檔案不存在，**默默繼續**。不要特別指出缺漏，也不要主動建議建立。`/domain-modeling` skill（經由 `/grill-with-docs` 與 `/improve-codebase-architecture`）會在術語或決策實際被定案時才按需建立。

## 檔案結構

Single-context repo：

```
/
├── GLOSSARY.md
├── docs/adr/
│   ├── 0001-....md
│   └── 0002-....md
├── frontend/
└── src-tauri/
```

## 使用 glossary 的詞彙

當輸出提到某個領域概念（issue 標題、重構提案、假設、測試名稱），使用 `GLOSSARY.md` 定義的術語，不要漂移到 glossary 明確避開的同義詞。

若需要的概念尚未出現在 glossary，這是一個訊號：可能你在發明專案沒用的語言（重新考慮），或真的有缺口（留給 `/domain-modeling` 處理）。

## 標示 ADR 衝突

若輸出與既有 ADR 矛盾，明確指出，不要默默覆蓋：

> _與 ADR-0007（event-sourced orders）矛盾，但值得重新討論，因為……_
