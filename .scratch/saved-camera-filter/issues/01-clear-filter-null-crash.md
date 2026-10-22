# 快速篩選的清除鈕無效，清單卡在篩選結果

Status: ready-for-agent

## 問題

「已儲存攝影機」清單上方的「快速篩選」輸入框，按右側清除鈕（X）沒有反應：輸入框文字不變，清單也維持篩選後的結果。之後若點擊「搜尋結果」中的攝影機開啟「新增攝影機」對話框，輸入框文字才會變成空白，但「已儲存攝影機」清單仍是舊的篩選結果。

## 重現步驟

1. 已儲存 3 台攝影機（前門攝影機、後院攝影機、車庫 WebCam）
2. 在快速篩選輸入「前門」，清單剩 1 台
3. 按輸入框右側的清除鈕
4. 觀察：輸入框仍顯示「前門」，清單仍是 1 台
5. 到「搜尋結果」點一台攝影機，開啟「新增攝影機」對話框
6. 觀察：輸入框變成空白，但「已儲存攝影機」清單仍是 1 台

## 根因

- Vuetify 的 `clearable` 清除鈕會呼叫 `reset()`，把 `modelValue` 設為 `null`（`vuetify/lib/composables/validation.js`）。
- `filteredSavedCameras` 直接呼叫 `savedCameraFilter.value.trim()`，收到 `null` 時拋出 `TypeError: Cannot read properties of null (reading 'trim')`，`HomeView` 的 render 中止，畫面停在最後一次成功的結果。
- 之後的 render 沿用拋錯前快取的篩選陣列，所以清單持續顯示篩選結果；輸入框則在下一次成功 render 時把 `null` 顯示為空字串。這解釋了「清除當下沒反應、開對話框時文字才消失」的順序。

## 修正

`frontend/src/views/HomeView.vue`：`savedCameraFilter` 改為 writable computed，setter 將 `null` 正規化為 `""`，維持「篩選字串永遠是 string」的不變式；`filteredSavedCameras` 不需再防 `null`。

## 驗收條件

- [ ] 有篩選字串時按清除鈕：輸入框清空，清單顯示全部已儲存攝影機
- [ ] 清除後失焦再聚焦：輸入框保持空白，清單維持全部
- [ ] 清除後開啟「新增攝影機」對話框再關閉：輸入框與清單狀態一致，不會出現「空白輸入框 + 篩選後清單」
- [ ] 沒有符合條件的篩選顯示「找不到符合篩選條件的攝影機」；完全沒有攝影機時顯示「尚無儲存的攝影機」
- [ ] 有篩選字串時失焦再聚焦，文字與清單維持不變
- [ ] `pnpm build`（vue-tsc）通過

## Comments

- 2026-10-22：修正已套用於工作目錄，上述驗收條件以 headless Chrome 載入 dev server 逐項驗證通過；待使用者在 Tauri 視窗中確認。
