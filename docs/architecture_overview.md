# Architecture Overview

## Dioxus-Desktop vs Tauri

This project uses Dioxus-Desktop as its primary framework for building the desktop application. Here's why:

### Dioxus-Desktop Advantages
1. **Rust-First Development**: Dioxus allows us to write the entire application in Rust, leveraging its safety and performance benefits.
2. **Hot Reloading**: Excellent development experience with hot reloading support.
3. **Cross-Platform**: Builds for Windows, macOS, and Linux from a single codebase.
4. **Lightweight**: Smaller binary size compared to Electron-based solutions.
5. **Modern UI**: Built on top of WebView2 (Windows), WKWebView (macOS), and WebKitGTK (Linux).

### Why Not Tauri?
While Tauri is also an excellent choice for desktop applications, we chose Dioxus-Desktop because:
1. **Simpler Architecture**: Dioxus provides a more straightforward development model for our use case.
2. **Rust-Centric**: Dioxus is more focused on Rust development, which aligns with our goals.
3. **Lower Overhead**: Dioxus has a smaller runtime footprint, which is beneficial for our performance requirements.

## Project Structure

```
.
├── src/                # Rust source code
│   ├── main.rs        # Application entry point
│   ├── components/    # Reusable UI components
│   └── utils/         # Utility functions
├── assets/            # Static assets
├── docs/              # Documentation
└── tests/             # Test files
```

## Key Components

1. **UI Layer**: Built with Dioxus components and Tailwind CSS for styling
2. **State Management**: Uses Dioxus hooks for local state and signals for reactive updates
3. **File System**: Native file system operations through Rust's standard library
4. **Tokenization**: Integration with tiktoken-rs for GPT-style token counting 