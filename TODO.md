# TODO

## Functionality
- [~] Cut, Copy and Paste
        - Use the hacks where a focused textarea is created and the pasted text will
            retrived from there.
            https://stackoverflow.com/questions/2787669/get-html-from-clipboard-in-javascript
    - [X] Replace, Paste to a selected region
           - The selected region is removed and the clipboard content is inserted
    - [ ] Increase the selection when shift key is pressed
    - [ ] Pressing any key will delete the selected text.
- [X] Move the cursor to the user clicked location
- [X] Redo/undo
    - [X] Undo/Redo on typing
    - [X] Undo/redo on cut/copy/paste
        - [X] Move to
        - [X] Add `join_line` to undo break line
    - [X] Select All
    - [X] Undo for deleted text selection
    - [X] Redo for deleted text selection
- [ ] multi-layer buffer, so you can drag objects around
- [X] Draggable separator
- [ ] selection
    - select parts of the text buffer
    - [ ] Show selected text
    - [X] Block selection
    - [ ] Use different background color for selected text
        - block mode
            - if the point is inside the rectangular selection
        - normal mode
            - if the point in in between lines - true
            - if the point is after start in the first line - true
            - if the point is before the end in the last line - true
- [ ] Scroll the view to when the cursor is not visible on the screen
    - This is done automatically due the textarea being always on the cursor, which is focused all the time
        and the browser automatically scroll it to view
- [X] Centralize the commands into actions first, so we can send an external messages
    and hook it into history for undor/redo functionality
    - the command functions are now returning effects, and its up for the caller to wire them.
        It doesn't need to be the centralized inm the update function as there could be many more variation of commands
- [ ] Render only lines that are visible to the user, we can calculate the line exposed to the users
- [X] Add an api to group action together in Recorded
        ```rust
        pub struct Recorded{
            history: VecDeque<Vec<Action>>,
            undone: VecDeque<Vec<Action>>,
        }
        ```
- [ ] Context Menu (Right click)
    - undo/redo
    - cut/copy/paste
- [~] Add config to ignore mouse-clicks outside of the editor range
- [~] Move the core-functionality of ultron to ultron, which has no view, just text manipulation
    - [X] ultron-core will have the following modules
        - [X] TextBuffer
        - [X] TextEdit (text-buffer with history)
        - [X] Editor
        with no dependency on sauron, all types is abstracted
    - The ultron is the bare minimum library, which relies on sauron
        - [X] Wrap the editor with sauron for the web
        - [ ] ultron-tui abstract terminal events into ultron events
- [ ] Add a throttle
    - record the last time the function is executed
        - get the elapsed time since the function is executed.
            - if lesser than the allowable, don't execute yet and mark it pending/dirty
                - then make a delayed call to the function
            - if the elapsed is more than the allowable, then execute the function, mark pending/dirty = false
            - if the last record is not yet there, then execute the function and set it to now
- [X] Always make the cursor visible, by scrolling to where the cursor is located
- [ ] Activating a context menu interferes with the selection since both are based on mouse clicks
- [X] Make the editor a custom_element using tag `ultron-editor`.
    Example usage
    ```html
        <ultron-editor value="The long text the user wants to be edited" on_change= {(|ie| {log::info!("The value changed to: {}", ie.value);})}/>
    ```
- [ ] Make an overridable keypress on the web_editor container

## Features
- [~] Smart edit blockmode
    - [X] When typing a key and the next characters next to it is far, say more than 2 space the character is typed in replace mode
        instead of insert mode
    - [ ] Pressing enter should indent, instead of just moving down
- [X] Allow the editor to render different syntax highlighting scheme to a set of lines
    - Use would be markdown text with code fence in the content
- [X] Make the keypresses be translated into Commands, so we can use remap such as vi, kakune, etc.
- [ ] Make ultron-widgets package to contain common widgets such as
    - [ ] toolbar
    - [ ] menu
        - [ ] context menu
    - [ ] status line

## Maintenance
- [X] Put the css of each of the component to their own module
    - line, range, cell
- ~~[ ] Make Line, Range and Cell implement View~~
     - This is not possible since these struct has a custom view function which neeeds access to multiple arguments
- [~] Render only the lines that are visible in the viewport
    - [X] Determine the number of lines that could fit in the viewport.
        ```
         let n_lines = viewport_height / line_height;
        ```
    - [ ] Group lines into page, determine if the page is visible in the viewport using the scroll top offset
          ```
            let page_size = 20; // how many lines in a page, could also be `n_lines` derived from how many lines that could fit in the viewport
            let page_height = page_size * line_height;
          ```
        - each page has an offset from the top
          ```
            let page_offset = page_n * page_height;
            if scroll_top > page_offset && scroll_top < next_page_offset { show } else {hide}
          ```
- [X] Simplify the text buffer, just a `Vec<Vec<char>>`, no paging, no ranges, no char.
- [ ] Make `ultron-ssg` have it's own rendering algorithm

## Issues
- [ ] Moving around characters with cell_width of 2
        should not need to pressed the arrow right 2 times to get ot the next character.
        This is because movement is merely adding +1 to x or y position.
        - [ ] Need to make cursor movement line cell aware.
- [ ] When typing very fast, the content in the textarea is accumulated and there is a duplicate on the characters
- [ ] Clicking outside of the editor will affect the editor, this is because we need to let the editor interact to mouse up drags
    when shapes are drawn into the canvas.
    - A solution would be to send the WindowMouseUp event, but not the window click event
- Putting the cursor before a quote and typing will make the cursor disappear
    - This might at before any range
- [X] The long svgbob example is taking a long time to update.
    - 180ms when syntax highlighting is enabled
        - scrolling is 20ms
        - building the view: 50ms
        - patching : 40ms
    - 80ms when syntax highlighting is disabled
        - Update: 15ms when syntax highlighting is disablled
        - Update: 50ms when syntax highlighting is enabled
- [X] When `replace_char` is called for the next page, it applies to the first page instead
- [X] replace addition and subtraction operation with saturating_add and saturating_sub.
- [X] If the top level view of a Program changes, then the original root_node is not set, which causes
    - [X] Add a test for replacing the top-level root-node and confirm it is changed
- [ ] When using the selection tool, the app will just crash due to borrowing error (Rc<RefCell>)
- [ ] Cursor color and gutter color and number has wrong pairing
    - [ ] Detect the color selected color from the theme and check to see if it is light or dark
        - contrast the color for the cursor number line and gutter
- [X] Avoid FOUC by waiting for the fonts to be loaded before mounting the application, this way the calculation of pixels from font size would be accurate

## Tests
- [ ] Add test for undo and redo (editor)

## Performance improvement
- [X] For every last typed character, add it to the most probable range in the highlighted line
    - It will be corrected once the highlighting is run
- [ ] Test to see if the CACHED_ELEMENT and the cached WINDOW and DOCUMENT in sauron is an improvement
- [ ] Make a runnable benchmark which replays typing the characters in the editor
- [X] Put the background hightlighting task inside of spawn_local
- [ ] Add a key to each line which is their respective line numbers.
    This way, it will be faster to match existing lines.
    - If a line is added, it will take a new key which appends ".1", thus it's key value will be "{n}.1".
    - If a new line is added to that just added line, we will append the same ".1".
```rust
1. The quick brown
    1.1 ,crazy
    1.1.1 local water
2. fox jumps over
3. the lazy dog!
```
- [ ] Backspace(DeleteBack) is not fast enough
- [ ] Delete(DeleteForward) is not fast enough
- [ ] Visual Cue for Tab is also slow

