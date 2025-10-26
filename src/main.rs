/// FILE: src/main.rs
///
/// This is the entry point of our application. When you run `cargo run`, execution
/// starts at the `main()` function below.
///
/// RUST CONCEPTS DEMONSTRATED:
/// - Module system: Using `mod` to declare modules from other files
/// - Result<T, E>: Rust's type for operations that can succeed (Ok) or fail (Err)
/// - Error propagation: Using `?` operator to bubble up errors
/// - NativeOptions: Configuration struct for the eframe window

// ============================================================================
// MODULE DECLARATIONS
// ============================================================================
// The `mod` keyword tells Rust to look for these modules in separate files:
// - `mod app` → looks for src/app.rs
// - `mod storage` → looks for src/storage.rs
// - `mod parser` → looks for src/parser.rs
//
// This keeps our code organized and maintainable.

mod app;
mod storage;
mod parser;

// ============================================================================
// MAIN FUNCTION - PROGRAM ENTRY POINT
// ============================================================================

/// The main function is where program execution begins.
///
/// Return type: Result<(), eframe::Error>
/// - Ok(()) means success (the unit type () is like void in other languages)
/// - Err(eframe::Error) means something went wrong during window setup
///
/// The `-> Result<(), eframe::Error>` syntax is Rust's way of saying
/// "this function might fail, and if it does, here's the error type."
fn main() -> Result<(), eframe::Error> {
    // ------------------------------------------------------------------------
    // WINDOW CONFIGURATION
    // ------------------------------------------------------------------------
    // NativeOptions is a struct that configures our application window.
    // We use a struct initialization syntax with named fields.
    let options = eframe::NativeOptions {
        // viewport_builder configures the initial window appearance
        viewport: egui::ViewportBuilder::default()
            // Set the initial window size to 1024x768 pixels
            .with_inner_size([1024.0, 768.0])
            // Set the minimum window size to prevent it from being too small
            .with_min_inner_size([400.0, 300.0])
            // Set the window title that appears in the title bar
            .with_title("BookScript Writer"),

        // Use default values for all other NativeOptions fields
        ..Default::default()
    };

    // ------------------------------------------------------------------------
    // APPLICATION LAUNCH
    // ------------------------------------------------------------------------
    // eframe::run_native is the function that:
    // 1. Creates the OS window
    // 2. Sets up the rendering context (graphics)
    // 3. Starts the event loop (handling input, drawing frames)
    //
    // Parameters:
    // - "BookScript Writer": Internal app name (for native integrations)
    // - options: The window configuration we created above
    // - Box::new(|cc| ...): A closure that creates our App instance
    //
    // OWNERSHIP NOTE:
    // Box::new allocates our app on the heap (not the stack) and gives
    // eframe ownership of it. eframe will keep the app alive until the
    // window is closed.
    eframe::run_native(
        "BookScript Writer",
        options,
        // This closure is called once when the app starts
        // `cc` (CreationContext) gives us access to egui integration info
        Box::new(|cc| {
            // Create and return our App instance
            // `Ok(Box::new(...))` means "successfully created the app"
            // The ? operator would propagate any errors from App::new()
            Ok(Box::new(app::App::new(cc)))
        }),
    )
    // The `?` operator here means: "if run_native returns an error, return
    // that error from main() immediately. Otherwise, continue."
}

// ============================================================================
// HOW THIS WORKS - THE EVENT LOOP
// ============================================================================
//
// Once eframe::run_native is called, here's what happens:
//
// 1. CREATE: The closure we passed creates our App instance
// 2. LOOP: eframe enters an infinite loop that runs ~60 times per second
// 3. UPDATE: Each frame, it calls app.update() to rebuild the UI
// 4. RENDER: egui draws the UI to the screen
// 5. REPEAT: Go back to step 2 until the window is closed
//
// This is called the "event loop" or "game loop" pattern.
// ============================================================================
