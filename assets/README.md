把支持中文的字体文件放到这个目录（可选）。

程序启动时会优先尝试加载：
- `assets/NotoSansSC-Regular.ttf` / `assets/NotoSansSC-Regular.otf`
- `assets/msyh.ttf`

如果没有放字体文件，程序会尝试从系统字体目录加载：
- Windows（例如 `C:/Windows/Fonts/simhei.ttf`）
- macOS（例如 `/System/Library/Fonts/PingFang.ttc`）
