# 已儲存攝影機清單改用 IPv4 數值排序

Status: ready-for-agent

## 問題

`storage.rs` 以 `name.to_lowercase()` 排序，而攝影機名稱預設就是 IP，所以 `10.1.6.100` 會排在 `10.1.6.99` 前面。另外 `add_camera` 只把新項目 push 到最後、`update_camera` 原地更新，兩者回傳的清單都沒有重排，新增或改 IP 後順序會不正確，直到重新載入。

## 變更

- 新增 `sort_cameras(&mut [CameraInfo])`：合法 IPv4 依數值排序，同 IP 再依 `onvif_port` 升冪；無法解析為 IPv4 的位址以字串（不分大小寫）排序並排在最後。
- `load_cameras`、`add_camera`、`update_camera` 都套用同一個排序，四個 storage 指令回傳的順序一致。
- 前端不需要改動。

## 驗收條件

- [x] `10.1.6.1` → `10.1.6.9` → `10.1.6.10` → `10.1.6.99` → `10.1.6.100`（數值順序）
- [x] 同 IP 不同 port：`10.1.6.1:80` 在 `10.1.6.1:8000` 之前
- [x] 非 IPv4（例如 `camera.local`）排在所有合法 IP 之後
- [x] 名稱與 IP 順序相反時，仍以 IP 排序（`storage::tests::sorts_by_numeric_ipv4_then_port`）
- [x] `cargo test` 通過
- [x] 新增、編輯、刪除後回傳的清單維持排序
- [x] 「搜尋結果」清單不受影響（維持掃描順序）

## Comments

- 2026-10-22：已實作於 `src-tauri/src/storage.rs`，單元測試涵蓋數值排序、port tie-break、非 IPv4 與名稱干擾情境。
