# Tauri Lazy Render Demo (純 Rust + 原生 HTML/JS 延遲載入範例)

本專案為基於 **Tauri** 與 **Rust** 建構的桌面應用程式效能示範專案。採用純前端原生技術（HTML / CSS / Vanilla JS）搭配 Rust 後端，展示如何在不依賴重型前端框架的情況下，透過「雙向滑動視窗（Sliding Window）」與「卷軸事件觸發分批渲染」，流暢載入大量訂單資料並維持極低的 DOM 節點數量。

## ✨ 核心特色

* **純 Rust 生態與原生前端**：零 Node.js / TypeScript 編譯依賴，直接使用 `cargo tauri dev` 啟動，透過 `window.__TAURI__.core.invoke` 與 Rust 後端通訊。
* **本地端模擬 API 資料來源**：透過 Rust 後端非同步讀取 `src-tauri/order_list.json`，模擬從後端伺服器取得大批訂單資料的情境。
* **雙向滑動視窗（Sliding Window）DOM 回收機制**：畫面最多僅維持約 2 個批次（預設 `visibleCount * 2 = 100` 筆）的 DOM 節點，滾動時動態卸載離開視野的舊節點，有效避免大量訂單圖片與明細造成的瀏覽器渲染卡頓。
* **防抖動（Debounce）卷軸偵測**：監聽表格容器的 `scroll` 事件，精準偵測觸頂與觸底時機，自動載入前一批或後一批資料並校正卷軸位置。

## 🛠️ 環境需求 (Prerequisites)

本專案為純 Rust 模板，無需安裝 Node.js 或 npm，僅需準備以下開發環境：

* **Rust 工具鏈**：`1.91.0` 或以上版本（透過 [rustup](https://rustup.rs/) 安裝）
* **Tauri CLI**：透過 Cargo 安裝 `cargo install tauri-cli`
* **系統編譯依賴**：
  * **Windows**：[Microsoft C++ Build Tools](https://visualstudio.microsoft.com/zh-hant/visual-cpp-build-tools/)（需勾選「使用 C++ 的桌面開發」）與 WebView2（Windows 10/11 通常已內建）。
  * **macOS**：Xcode Command Line Tools (`xcode-select --install`)。
  * **Linux (Ubuntu/Debian)**：`libwebkit2gtk-4.1-dev`、`build-essential`、`curl`、`wget`、`file`、`libxdo-dev`、`libssl-dev`、`libayatana-appindicator3-dev`、`librsvg2-dev`。


可以直接用VS Code開啟此專案，自動安裝 tauri-cli 和rust 工具，但其他的相依工具依然需要安裝。

## 🚀 快速開始 (Getting Started)



### 1. 取得專案

```bash
git clone https://github.com/0Speechless0/lazy-render-demo.git
cd lazy-render-demo
```

### 2. 啟動開發模式 (Development)

直接在專案根目錄執行 Tauri CLI 指令：

```bash
cargo tauri dev
```

> **提示**：首次編譯 Rust 依賴套件（Crates）需要較長時間，後續增量編譯將會非常快速。

### 3. 打包建置正式版本 (Production Build)

```bash
cargo tauri build
```

建置完成的安裝檔與執行檔將輸出至 `src-tauri/target/release/bundle/` 目錄下。

## 📁 專案結構 (Project Structure)

```text
lazy-render-demo/
├── src/                        # 原生前端靜態資源目錄
│   ├── assets/                 # 前端靜態圖片、圖示與媒體資源
│   ├── index.html              # 主畫面結構與雙向分批渲染邏輯 (Vanilla JS)
│   └── styles.css              # 全域與表格樣式表
└── src-tauri/                  # Rust 後端核心程式碼與設定
    ├── icons/                  # 應用程式編譯打包後使用的各平台圖示 (icns, ico, png)
    ├── src/
    │   └── main.rs             # Rust 進入點與 `read_file` 指令實作
    ├── order_list.json         # 模擬從後端 API 取得的本地訂單 JSON 資料集
    ├── Cargo.toml              # Rust 套件依賴設定
    └── tauri.conf.json         # Tauri 視窗與應用程式設定檔
```

## 💡 延遲載入與雙向分批渲染實作說明

本專案在 `src/index.html` 中針對 `#scroll-container`（外層固定高度 `max-height: 800px` 的滾動容器）與 `#inner-container`（`<tbody>`）實作了動態資料視窗管理：

1. **初始資料讀取 (`invoke("read_file")`)**
   * 頁面載入時呼叫 `loadOrders(500)`，透過 Tauri IPC 呼叫 Rust 後端的 `read_file` 指令讀取 `src-tauri/order_list.json` 並解析為 `orders` 陣列。
   * 首次呼叫 `render(0)` 渲染第 `0` 批資料（預設 `visibleCount = 50` 筆）。

2. **卷軸滾動偵測與防抖動 (`scroll` + `debounce`)**
   * 為避免滾動過程中頻繁觸發重繪，監聽器包覆了 `300ms` 的 `debounce` 函式，並透過 `rendering` 狀態鎖防止重複進入。
   * **向下滾動觸底（載入後一批資料）**：
     * 判斷條件：`scroll.scrollTop + scroll.clientHeight >= scroll.scrollHeight`
     * 行為：將批次索引 `sectionIndex` 遞增（`++sectionIndex`），呼叫 `render(sectionIndex * visibleCount, true)` 將下一批 50 筆資料透過 `tbody.appendChild(tr)` 附加至表格底部。
     * **DOM 節點回收**：當 `tbody.childElementCount > visibleCount * 2`（超過 100 筆）時，自動從頂部移除最舊的 50 個 `<tr>` 節點，保持 DOM 樹輕量化。
   * **向上滾動觸頂（重新讀取前一批資料）**：
     * 判斷條件：`scroll.scrollTop === 0` 且 `sectionIndex > 0`
     * 行為：將批次索引 `sectionIndex` 遞減（`--sectionIndex`），呼叫 `render(sectionIndex * visibleCount, false)`。
     * **DOM 節點回收與反向插入**：先從 `tbody` 尾部移除超出的舊節點，再將前一批資料切片（`slice(startIdx - visibleCount, startIdx)`）反轉（`.reverse()`）後透過 `tbody.prepend(tr)` 依序插回表格最上方。

3. **卷軸位置校正（Scroll Position Anchoring）**

   * **運作原理**：每次觸發觸頂或觸底載入並抽換節點後，程式會強制執行 `scroll.scrollTop = scroll.scrollHeight / 2`。

   * **為什麼是 `scrollHeight / 2`（視覺錨點對齊）**：由於容器中固定維持「**上一批（50 筆）＋ 當前批（50 筆）**」共 100 筆節點，`scrollHeight / 2` 恰好對應這兩批資料的交界處。

     * **向下觸底載入後**：原本在底部的第 `N` 批資料會被推升到上半部，新載入的第 `N+1` 批佔據下半部；將卷軸設為一半，使用者視線會無縫銜接在兩批資料的交界處，且上下皆留有 50 筆的滾動緩衝空間。

     * **向上觸頂載入後**：若不校正卷軸，新插入在頂部的節點會將原本正在閱讀的列往下推，導致 `scrollTop` 依然停留在 `0` 而引發連續觸頂誤判。設為 `scrollHeight / 2` 可將視角精準定錨回使用者原本滾動前的畫面位置。

4. **為什麼 `visibleCount` 不等於 `tbody.childElementCount`？**

   * **概念區分（單次渲染單位 vs. 實際 DOM 緩衝池上限）**：

     * `visibleCount`（預設 `50`）代表的是「**單次批次增量載入的資料筆數（Batch Size）**」，並非整個表格的總節點上限。

     * `tbody.childElementCount` 則是 DOM 樹中「**實際存活的 `<tr>` 總數**」。為了實現流暢的雙向滑動視窗（Sliding Window），系統必須同時保留「**目前批次**」與「**相鄰的緩衝批次**」，因此常態下的節點總數上限為 **`visibleCount * 2`（100 筆）**。

   * **生命週期中的動態數量變化**：

     1. **初始狀態（第 0 批）**：剛載入時僅渲染第一批，此時 `childElementCount` 為 `50`（等於 `visibleCount`）。

     2. **第一次觸底（載入第 1 批）**：附加 50 筆後 `childElementCount` 變為 `100`；因為尚未超過 `visibleCount * 2`（`100 > 100` 為 `false`），不會觸發回收，此時 `childElementCount` 維持在 `100`（`visibleCount` 的兩倍）。

     3. **後續雙向滾動**：每次新增 50 筆（暫時達到 150 筆）後，回收機制會立即切除另一端的舊節點，使 `childElementCount` 穩定維持在 `100` 筆（若滾動至最後一頁且剩餘資料不足 50 筆，或往上回捲時的邊界切片計算 `i > topIdx`，則會落在 `50 ~ 100` 筆之間）。

## 📄 授權條款 (License)

本專案採用 [MIT License](LICENSE) 授權。