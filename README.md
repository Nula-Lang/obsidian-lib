# Obsidian Lib 

# Architecture
+-----------------------------------+
|          Aplikacja Nula           |
| (kod .nula → kompilowany do bin)  |
+-----------------------------------+
              │
              ▼
+-----------------------------------+
|          Obsidian Lib             |
| (Część Rust → native lib)  |
| - API GUI (Nula bindings)         |
| - Tauri bridge                    |
| - HTML/CSS renderer (via WebView) |
+-----------------------------------+
              │
              ▼
+-----------------------------------+
|        Frontend (HTML/CSS/JS)     |
|        - layout GUI               |
|        - logika w JS (opcjonalna) |
+-----------------------------------+
