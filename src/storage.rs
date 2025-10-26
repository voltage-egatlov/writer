use anyhow::{Context, Result};
/// FILE: src/storage.rs
///
/// This module handles all file I/O operations and autosave functionality.
///
/// RUST CONCEPTS DEMONSTRATED:
/// - std::fs: File system operations (reading, writing files)
/// - std::path: Cross-platform path handling
/// - anyhow: Flexible error handling with context
/// - std::thread::sleep: Pausing execution
/// - std::time::Duration: Representing time intervals
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// ============================================================================
// FILE I/O FUNCTIONS
// ============================================================================

/// Load text content from a file on disk
///
/// PARAMETERS:
/// - `path`: A reference to anything that can be used as a Path
///   (PathBuf, Path, String, &str, etc.)
///   The `AsRef<Path>` trait bound means "accept any type that can be
///   treated as a Path reference"
///
/// RETURN TYPE:
/// - Result<String, anyhow::Error>
///   Success: Ok(String) containing the file contents
///   Failure: Err(Error) with context about what went wrong
///
/// ERROR HANDLING:
/// The `?` operator propagates errors up the call stack. If any operation
/// fails, we immediately return Err(...) to the caller.
/// The `.context()` method adds human-readable context to errors.
pub fn load_text_file<P: AsRef<Path>>(path: P) -> Result<String> {
    // Convert the generic path parameter to a Path reference
    let path = path.as_ref();

    // fs::read_to_string reads the entire file into a String
    // The ? operator means: "if this returns Err, return that error immediately"
    // .context() adds context so the user knows WHICH operation failed
    let content =
        fs::read_to_string(path).context(format!("Failed to read file: {}", path.display()))?;

    // If we got here, reading succeeded
    // Return Ok(content) wrapped in Result
    Ok(content)
}

/// Save text content to a file on disk
///
/// PARAMETERS:
/// - `path`: Where to save the file
/// - `content`: What to write (a string reference)
///   `&str` is a string slice - a view into string data
///   It doesn't own the string, just borrows it
///
/// RETURN TYPE:
/// - Result<()>: Success returns Ok(()), failure returns Err(Error)
///   The unit type `()` is like void - it means "no meaningful return value"
pub fn save_text_file<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    let path = path.as_ref();

    // Before saving, ensure the parent directory exists
    // Example: if path is "/foo/bar/file.txt", we need "/foo/bar" to exist
    if let Some(parent) = path.parent() {
        // fs::create_dir_all creates all missing parent directories
        // Like `mkdir -p` in Unix/Linux
        fs::create_dir_all(parent)
            .context(format!("Failed to create directory: {}", parent.display()))?;
    }

    // fs::write writes the entire string to a file
    // If the file exists, it's overwritten
    // If it doesn't exist, it's created
    fs::write(path, content).context(format!("Failed to write file: {}", path.display()))?;

    // Success!
    Ok(())
}

/// Get the path to the autosave directory
///
/// On Windows: C:\Users\USERNAME\AppData\Roaming\BookScript\projects
/// On Linux: ~/.config/BookScript/projects
/// On macOS: ~/Library/Application Support/BookScript/projects
///
/// RETURN TYPE:
/// - Result<PathBuf>: A newly allocated path buffer
///   PathBuf is like String, but for file paths
///   Path is like &str, but for file paths
///
/// ERROR HANDLING:
/// If we can't determine the user's data directory, we return an error
pub fn get_autosave_dir() -> Result<PathBuf> {
    // directories::ProjectDirs finds the appropriate directories for our app
    // "com", "BookScript", "BookScript" are:
    // - Qualifier (company/organization)
    // - Organization name
    // - Application name
    //
    // These create a unique namespace: com.BookScript.BookScript
    let proj_dirs = directories::ProjectDirs::from("com", "BookScript", "BookScript")
        .context("Could not determine user data directory")?;

    // data_dir() gives us the main data directory
    // We append "projects" to store our autosave files there
    let autosave_dir = proj_dirs.data_dir().join("projects");

    // Ensure the directory exists before returning
    fs::create_dir_all(&autosave_dir).context(format!(
        "Failed to create autosave directory: {}",
        autosave_dir.display()
    ))?;

    Ok(autosave_dir)
}

// ============================================================================
// AUTOSAVE THREAD FUNCTION
// ============================================================================

/// Background thread that periodically saves the document
///
/// This function runs in a separate thread and loops forever, waking up
/// every 60 seconds to save the current text content.
///
/// PARAMETERS:
/// - `text_content`: Arc<Mutex<String>> shared with the GUI thread
///   Arc allows multiple threads to own the same data
///   Mutex ensures only one thread accesses it at a time
///
/// THREADING SAFETY:
/// The Mutex ensures that when we lock and read the text, the GUI thread
/// can't modify it at the same time. This prevents data races.
///
/// INFINITE LOOP:
/// This function never returns - it runs until the program exits.
/// When the main thread (GUI) exits, all background threads are terminated.
pub fn autosave_thread(text_content: Arc<Mutex<String>>) {
    // This loop runs forever
    loop {
        // Sleep for 60 seconds
        // Duration::from_secs(60) creates a 60-second time interval
        // thread::sleep pauses this thread without consuming CPU
        thread::sleep(Duration::from_secs(60));

        // After waking up, perform the autosave

        // ----------------------------------------------------------------
        // STEP 1: Get the autosave directory path
        // ----------------------------------------------------------------
        let autosave_dir = match get_autosave_dir() {
            Ok(dir) => dir,
            Err(e) => {
                // If we can't get the directory, print an error and skip this save
                eprintln!("Autosave error: {}", e);
                // `continue` jumps back to the start of the loop
                continue;
            }
        };

        // ----------------------------------------------------------------
        // STEP 2: Create the autosave file path
        // ----------------------------------------------------------------
        // We save to "autosave.bks" in the autosave directory
        let autosave_path = autosave_dir.join("autosave.bks");

        // ----------------------------------------------------------------
        // STEP 3: Lock the mutex and clone the text content
        // ----------------------------------------------------------------
        // IMPORTANT: We clone the string so we can release the lock quickly
        // Holding the lock during file I/O would block the GUI thread
        let content = {
            // Lock the mutex - this blocks if the GUI thread is holding the lock
            // The lock is automatically released when `guard` goes out of scope
            let guard = text_content.lock().unwrap();
            // Clone the string (makes a copy of the text)
            guard.clone()
            // `guard` goes out of scope here, releasing the lock
        };

        // ----------------------------------------------------------------
        // STEP 4: Save to disk
        // ----------------------------------------------------------------
        match save_text_file(&autosave_path, &content) {
            Ok(_) => {
                // Success! Print a message (appears in the terminal)
                println!("Autosaved to: {}", autosave_path.display());
            }
            Err(e) => {
                // Error! Print to stderr
                eprintln!("Autosave failed: {}", e);
            }
        }

        // Loop continues - wait another 60 seconds and repeat
    }
}

// ============================================================================
// HOW THREADING WORKS IN THIS MODULE
// ============================================================================
//
// THREAD RELATIONSHIPS:
//
// Main Thread (GUI):                Autosave Thread:
//   |                                    |
//   | Creates Arc<Mutex<String>>         |
//   |-------------------------------->---|
//   | Clones Arc pointer                 |
//   | Spawns thread                      |
//   |                                    |
//   | Editing text...                    | Sleep 60s...
//   | (locks mutex)                      |
//   | Modifies string                    |
//   | (unlocks mutex)                    |
//   |                                    | Wake up!
//   | Drawing UI...                      | (locks mutex)
//   |                                    | Clone string
//   |                                    | (unlocks mutex)
//   | Editing text...                    | Save to disk...
//   | (locks mutex)                      |
//   | ...                                | Sleep 60s...
//
// MUTEX PREVENTS SIMULTANEOUS ACCESS:
// If both threads try to lock at the same time, one waits until the other
// releases the lock. This prevents data corruption.
//
// ============================================================================

// ============================================================================
// ERROR HANDLING PHILOSOPHY
// ============================================================================
//
// This module uses the `anyhow` crate for error handling:
//
// 1. Return Result<T, anyhow::Error> from functions that can fail
// 2. Use the ? operator to propagate errors up the call stack
// 3. Use .context() to add helpful error messages
// 4. The caller can decide how to handle errors (show to user, log, etc.)
//
// Example error chain:
//   Error: Failed to write file: /path/to/file.txt
//   Caused by: Permission denied (os error 13)
//
// This gives users actionable information about what went wrong.
// ============================================================================
