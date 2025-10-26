use crate::storage;
/// FILE: src/app.rs
///
/// This module contains our main App struct and implements the eframe::App trait.
/// The App trait requires one method: update(), which is called every frame (~60 fps).
///
/// RUST CONCEPTS DEMONSTRATED:
/// - Structs: Custom data types that group related data
/// - Traits: Interfaces that define behavior (like Java interfaces)
/// - impl blocks: Where we define methods on structs
/// - Mutable references (&mut): Allowing safe modification of data
/// - Arc<Mutex<T>>: Thread-safe shared ownership with interior mutability
use std::sync::{Arc, Mutex};
use std::thread;

// ============================================================================
// APP STRUCT - APPLICATION STATE
// ============================================================================

/// The App struct holds all the state for our application.
///
/// OWNERSHIP & THREADING:
/// - `text_content` is wrapped in Arc<Mutex<String>>
/// - Arc = "Atomic Reference Counted" = allows multiple owners
/// - Mutex = "Mutual exclusion" = ensures only one thread accesses at a time
/// - We need this because the autosave thread and GUI thread both access it
pub struct App {
    /// The text being edited by the user
    /// Arc<Mutex<String>> allows both the GUI thread and autosave thread
    /// to safely access the same string. When you want to read or write,
    /// you call .lock() to get exclusive access.
    text_content: Arc<Mutex<String>>,

    /// Path to the current project file
    /// Option<T> means "this might be Some(value) or None"
    /// We use None when no file is open yet
    current_file_path: Option<std::path::PathBuf>,

    /// Status message shown at the bottom of the window
    /// (e.g., "Autosaved at 14:23:45" or "File loaded successfully")
    status_message: String,
}

// ============================================================================
// IMPLEMENTATION - APP METHODS
// ============================================================================

impl App {
    /// Constructor for the App struct
    ///
    /// `cc` (CreationContext) is provided by eframe and contains info about
    /// the rendering context, storage, and integration settings.
    ///
    /// We mark it with underscore `_cc` to tell the compiler "we know we're
    /// not using this parameter yet, but we might need it later."
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Create a new empty String and wrap it in Arc<Mutex<>> for sharing
        // Arc::new() creates the reference-counted pointer
        // Mutex::new() creates the lock around the String
        let text_content = Arc::new(Mutex::new(String::new()));

        // Clone the Arc to create a second pointer to the same data
        // This doesn't clone the String itself, just the pointer!
        // Arc uses atomic reference counting to track how many pointers exist
        let text_for_autosave = Arc::clone(&text_content);

        // --------------------------------------------------------------------
        // SPAWN AUTOSAVE THREAD
        // --------------------------------------------------------------------
        // thread::spawn creates a new OS thread that runs concurrently
        // The thread runs the closure we pass to it
        // `move` keyword: the closure takes ownership of text_for_autosave
        thread::spawn(move || {
            // This code runs in a separate thread, independent of the GUI
            // Call our autosave function (defined in storage.rs)
            storage::autosave_thread(text_for_autosave);
            // When this function returns, the thread exits
        });

        // --------------------------------------------------------------------
        // RETURN THE APP INSTANCE
        // --------------------------------------------------------------------
        // `Self` is shorthand for `App` when inside an impl block
        // This creates and returns a new App instance
        Self {
            text_content,
            current_file_path: None,               // No file open initially
            status_message: String::from("Ready"), // Initial status
        }
    }

    /// Load a file from disk into the editor
    ///
    /// `&mut self` means this method borrows the App mutably
    /// (it can modify the App's fields)
    fn load_file(&mut self, path: std::path::PathBuf) {
        // storage::load_text_file returns Result<String, anyhow::Error>
        // We use pattern matching to handle both success and error cases
        match storage::load_text_file(&path) {
            // If loading succeeded, we get Ok(content)
            Ok(content) => {
                // Lock the mutex to get mutable access to the String
                // `.lock()` returns a MutexGuard<String>
                // `.unwrap()` panics if the lock is poisoned (very rare)
                // The `*` dereferences the guard to get the String itself
                *self.text_content.lock().unwrap() = content;

                // Update our state to remember which file is open
                self.current_file_path = Some(path.clone());

                // Update status message for the user
                self.status_message = format!("Loaded: {}", path.display());
            }
            // If loading failed, we get Err(e) where e is the error
            Err(e) => {
                // Show the error to the user in the status bar
                self.status_message = format!("Error loading file: {}", e);
            }
        }
    }

    /// Save the current text to a file on disk
    fn save_file(&mut self, path: std::path::PathBuf) {
        // Lock the mutex and clone the string contents
        // We clone because we need to keep the lock time short
        // (holding locks too long can cause performance issues)
        let content = self.text_content.lock().unwrap().clone();

        // Attempt to save the file
        match storage::save_text_file(&path, &content) {
            Ok(_) => {
                // Update our state
                self.current_file_path = Some(path.clone());
                self.status_message = format!("Saved: {}", path.display());
            }
            Err(e) => {
                self.status_message = format!("Error saving file: {}", e);
            }
        }
    }
}

// ============================================================================
// TRAIT IMPLEMENTATION - eframe::App
// ============================================================================

/// Implement the eframe::App trait for our App struct
///
/// TRAITS are Rust's way of defining shared behavior (like interfaces).
/// eframe requires us to implement the `update` method, which it calls
/// every frame to rebuild the UI.
impl eframe::App for App {
    /// Called by eframe each frame to build the UI
    ///
    /// Parameters:
    /// - `&mut self`: Mutable reference to our app (we can modify state)
    /// - `ctx`: The egui Context, which provides access to all UI widgets
    /// - `_frame`: Frame info (we don't use it, hence the underscore)
    ///
    /// IMMEDIATE MODE GUI:
    /// Unlike traditional GUI frameworks that maintain a tree of widgets,
    /// egui rebuilds the entire UI from scratch every frame. This might
    /// sound inefficient, but it's actually very fast and makes code simpler.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ====================================================================
        // TOP PANEL - MENU BAR
        // ====================================================================
        // TopBottomPanel creates a bar at the top of the window
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // `ui` is a Ui object that lets us add widgets
            // It's passed to us by the closure

            // Create a horizontal menu bar
            egui::menu::bar(ui, |ui| {
                // "File" menu
                ui.menu_button("File", |ui| {
                    // "Open" button
                    if ui.button("Open (.bks/.scr)").clicked() {
                        // In a real app, you'd use a file picker dialog here
                        // For now, we'll load a test file if it exists
                        let test_path = std::path::PathBuf::from("test.bks");
                        self.load_file(test_path);
                    }

                    // "Save As" button
                    if ui.button("Save As...").clicked() {
                        // In a real app, you'd use a file picker dialog
                        // For now, we'll save to a default location
                        let save_path = std::path::PathBuf::from("output.bks");
                        self.save_file(save_path);
                    }

                    // Separator line in the menu
                    ui.separator();

                    // "Exit" button
                    if ui.button("Exit").clicked() {
                        // ctx.send_viewport_cmd tells eframe to close the window
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                // "Help" menu
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.status_message =
                            String::from("BookScript Writer v0.1.0 - A simple writing app");
                    }
                });
            });
        });

        // ====================================================================
        // BOTTOM PANEL - STATUS BAR
        // ====================================================================
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            // Add some padding around the status message
            ui.add_space(4.0);

            // Display the status message
            ui.horizontal(|ui| {
                ui.label("Status:");
                ui.label(&self.status_message);
            });

            ui.add_space(4.0);
        });

        // ====================================================================
        // CENTRAL PANEL - TEXT EDITOR
        // ====================================================================
        // CentralPanel fills all remaining space after top/bottom panels
        egui::CentralPanel::default().show(ctx, |ui| {
            // Lock the mutex to get access to the text content
            // `.lock()` blocks until we can acquire the lock
            // `.unwrap()` panics if the mutex is poisoned
            let mut text = self.text_content.lock().unwrap();

            // Create a scrollable area that fills the available space
            egui::ScrollArea::vertical().show(ui, |ui| {
                // TextEdit::multiline creates a text editor widget
                //
                // `&mut *text` explanation:
                // - `text` is a MutexGuard<String>
                // - `*text` dereferences it to get &String
                // - `&mut *text` creates a mutable reference &mut String
                //
                // This is how we modify the string through the mutex guard
                ui.add(
                    egui::TextEdit::multiline(&mut *text)
                        // Make the editor fill all available space
                        .desired_width(f32::INFINITY)
                        .desired_rows(30)
                        // Use a monospace font (good for code/writing)
                        .font(egui::TextStyle::Monospace), // Show line numbers? (commented out for now)
                                                           // .code_editor()
                );
            });

            // The MutexGuard is automatically dropped here (goes out of scope)
            // This releases the lock so other threads can access the text
        });

        // ====================================================================
        // CONTINUOUS RENDERING
        // ====================================================================
        // By default, egui only redraws when there's user input
        // request_repaint() tells it to keep redrawing every frame
        // This is useful for animations or background updates like autosave
        ctx.request_repaint();
    }
}

// ============================================================================
// HOW THE GUI WORKS - FRAME BY FRAME
// ============================================================================
//
// Each frame (~16ms at 60 fps), here's what happens:
//
// 1. eframe calls update()
// 2. We build the UI by calling methods like TopBottomPanel::show()
// 3. egui records all the widget calls (buttons, text editors, etc.)
// 4. egui calculates layout (where everything should be positioned)
// 5. egui renders the UI to the screen using the graphics backend
// 6. User input (mouse clicks, keyboard) is processed
// 7. If anything changed, we update our App state
// 8. Repeat from step 1
//
// This is why it's called "immediate mode" - we immediately describe what
// the UI should look like right now, rather than building a widget tree
// that persists across frames.
// ============================================================================
