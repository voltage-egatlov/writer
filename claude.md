# Claude Context - BookScript Writer

**Last Updated:** 2025-10-26

## Project Overview

BookScript Writer is a Rust-based GUI text editor built with egui/eframe, designed for screenplay and script writing. The application features autosave functionality and is intended to support custom screenplay tags like `[CHAPTER: X]` and `[SCENE: Beach]`.

**Project Name:** writer_rust  
**Language:** Rust (Edition 2021)  
**Version:** 0.1.0  
**Target Rust Version:** 1.90.0 (September 2024)

## Architecture

### Core Components

1. **main.rs** - Entry point that launches the eframe window
2. **app.rs** - Main App struct implementing the eframe::App trait
3. **storage.rs** - File I/O and autosave functionality
4. **parser.rs** - Placeholder for future screenplay tag parsing (not yet implemented)

### Key Technologies

- **eframe 0.29** - Native GUI framework
- **egui 0.29** - Immediate-mode GUI library
- **directories 5.0** - Cross-platform user directory discovery
- **anyhow 1.0** - Error handling

## Current Features

### Implemented
- Multi-line text editor with monospace font
- Autosave every 60 seconds to `~/.config/BookScript/projects/autosave.bks` (Linux)
- Basic file operations (Open/Save As - currently using hardcoded paths)
- Thread-safe text storage using `Arc<Mutex<String>>`
- Status bar showing save/load operations
- Top menu bar with File and Help menus

### Planned (Not Yet Implemented)
- Screenplay tag parsing: `[CHAPTER: X]`, `[SCENE: Beach]`, `[ACT: I]`
- Document structure extraction (chapters, scenes, acts hierarchy)
- Table of contents/outline sidebar
- File picker dialogs for Open/Save
- Syntax highlighting for tags
- Tag validation and error reporting

## File Structure

```
writer_rust/
├── Cargo.toml              # Package manifest with dependencies
├── Cargo.lock              # Locked dependency versions
├── src/
│   ├── main.rs             # Entry point, window setup
│   ├── app.rs              # GUI implementation, App struct
│   ├── storage.rs          # File I/O, autosave thread
│   └── parser.rs           # Tag parsing (placeholder)
├── target/                 # Build output (gitignored)
└── writingtool/            # Unknown directory (needs investigation)
```

## Key Implementation Details

### Threading Model
- **Main Thread:** Runs the GUI event loop (~60 fps)
- **Autosave Thread:** Background thread that wakes every 60 seconds to save

### Data Sharing
- Text content stored in `Arc<Mutex<String>>` for thread-safe access
- Arc cloned for autosave thread (shared ownership)
- Mutex ensures mutual exclusion between GUI and autosave operations

### File Locations

**Autosave Directory:**
- Linux: `~/.config/BookScript/projects/`
- Windows: `C:\Users\USERNAME\AppData\Roaming\BookScript\projects`
- macOS: `~/Library/Application Support/BookScript/projects`

**Autosave File:** `autosave.bks`

## Important Code Locations

### App State (app.rs:20-30)
```rust
pub struct App {
    text_content: Arc<Mutex<String>>,
    current_file_path: Option<std::path::PathBuf>,
    status_message: String,
}
```

### Autosave Thread (storage.rs:91-145)
- Runs infinite loop with 60-second sleep
- Locks mutex, clones text, releases lock
- Saves to autosave.bks

### UI Update Loop (app.rs:101-200+)
- Top panel: Menu bar with File/Help menus
- Bottom panel: Status bar
- Central panel: Multiline text editor

## Development Notes

### Immediate Mode GUI Pattern
egui rebuilds the entire UI every frame (~60 fps). This is fast and simplifies state management compared to retained-mode GUIs.

### Error Handling Philosophy
- All I/O functions return `Result<T, anyhow::Error>`
- Use `.context()` to add human-readable error messages
- Errors displayed in status bar for user visibility

### Current Limitations
1. No file picker dialogs - uses hardcoded paths (test.bks, output.bks)
2. Parser module is just a placeholder with no functionality
3. No syntax highlighting or tag visualization
4. No document structure view
5. Autosave file not loaded on startup

## Next Steps / TODO

**High Priority:**
1. Add file picker dialogs for Open/Save (consider rfd crate)
2. Load autosave.bks on startup if it exists
3. Implement basic tag parsing in parser.rs

**Medium Priority:**
4. Add regex dependency for tag matching
5. Implement TagType parsing for CHAPTER/SCENE/ACT
6. Create sidebar with document outline
7. Add syntax highlighting for tags

**Low Priority:**
8. Tag validation and error reporting
9. Export to different formats
10. Undo/redo functionality
11. Search and replace

## Design Decisions

### Why Arc<Mutex<String>>?
Both GUI thread and autosave thread need access to text content. Arc provides shared ownership, Mutex ensures thread-safe access.

### Why 60-second autosave interval?
Balance between data safety and I/O overhead. Can be made configurable later.

### Why immediate-mode GUI (egui)?
- Simpler state management
- Fast performance despite rebuilding UI every frame
- Good Rust ecosystem support
- Cross-platform

## Known Issues
None currently - application is functional for basic text editing with autosave.

## Questions / Unknowns
- What is the `writingtool/` directory for?
- Should we support multiple file formats beyond .bks/.scr?
- Do we need collaborative editing features?
- Should autosave be configurable (interval, location)?

---

**Note to Claude:** This file serves as continuous context for the BookScript Writer project. Update it when making significant changes, architectural decisions, or discovering new information about the codebase.
