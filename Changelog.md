# Changelog

- Add a conventient function in text_highlighter to choose a theme while creating an instance of itself
- Add clear text selection function
- Select all for block mode
- Update doc comments, remove commented out code
- Add support for selecting all text, Fix delete order such that we delete from the last to the first to make the index still accurate near the start
- Fix join ling and cursor move afterwards
- Add support for undo/redo for breakline
- Add break_line function
- increase sleep time after publishing syntaxes-themes since it contains big dump file

## 0.2.5
- Make the gutter foreground in ayu-light have more contrast with its gutter
- Fix bounds checking in ultron, it has to be inside the edit, not necessary inside the textbuffer, which is problematic when there was no content in the textbuffer
- Add initial implementation for undo/redo
- Merge branch 'develop'
- Add History and Action structs needed for undo/redo
- Add RUSTFLAGS when for github ci
- Expose status line to allow apps to display it by themselves, show the number of msgs that was dispatched in the update loop
- Make the commands that modify text return a result triggering the on_change_listener of the editor

## 0.2.4
- Add ultron-ssg in README
- Display the average update time
- Add a preloader
- Fix the github rust workflow, swap the call sequence order for copy/cut text, try the most recent api first
- Improve the copy/cut for older browser such as webkit2, used in webview crate
- Make the code structure for copy and cut more robust
- Improve cursor placement when inserting text
- Improve copy pasting from the insert_text improvement
- Fix insert_text api and add tests
- Rearrange new test to the bottom
- Use a more descriptive name
- Implement cut_text for both block_mode and normal mode
- Remove log::trace here
- Fix calculation of editor_offset, we don;t include the window scroll
- Use a more descriptive Msg variant to indicate where the Msg is coming from

## 0.2.3
- minor release for ultron 0.2.2

## 0.2.1
- minor release for ultron without the feature flag
- Remove with-dom feature flag in text_buffer
- Add a feature flag for copy_to_clipboard
- increase sleep timer when running the publish script

## 0.2.0
- Add a utility function to normalize points
- Improve the api for replace_char
- limit the selections to 3
- Add an exec_copy as alternative to copy command
- Make copy and paste work
- Streamline the selection
- Fix ultron-ssg
- Use nalgebra to containts 2d points instead of overuse of tuples
- Add bounds check for the cursor
- Allow the Options passed in the Editor, syntax_token is now part of the options
- round the float when displayed
- Use span instead of div
- Fire the external messages when keypressed event was triggered
- Make lines use span when the options is specified, otherwise use the div. Use style 'min-width: max-content' to make the code more presentable when there are longer lines
- Add support for key composing
    - Example: In linux, `<CTRL>``<SHIT>``U+0024` will type in `$` in the editor.
- Display the average update time
- Add a preloader
- Implement cut/copy/paste
-   - Support for older browser such as webkit2 when used in webview
- Use nalgebra to containts 2d points instead of overuse of tuples
- Allow the Options passed in the Editor, syntax_token is now part of the options
- Use span instead of div
- Fire the external messages when keypressed event was triggered
- Use style 'min-width: max-content' to make the code more presentable when there are longer lines

## 0.1.2
- Performance improvements
- Syntax-highlighting

## 0.1.1
- First real release
