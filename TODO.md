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
- [ ] Redo/undo
    - [X] Undo/Redo on typing
    - [X] Undo/redo on cut/copy/paste
        - [X] Move to
        - [X] Add `join_line` to undo break line
    - [X] Select All
    - [ ] Undo for deleted text selection
    - [ ] Redo for deletex text selection
- [ ] multi-layer buffer, so you can drag objects around
- [X] Draggable separator
- [X] selection
    - select parts of the text buffer
- [ ] Scroll the view to when the cursor is not visible on the screen
    - This is done automatically due the textarea being always on the cursor, which is focused all the time
        and the browser automatically scroll it to view
- [ ] Centralize the commands into actions first, so we can send an external messages
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
- [ ] Add config to ignore mouse-clicks outside of the editor range
- [ ] Move the core-functionality of ultron to ultron, which has no view, just text manipulation
    - The ultron is the bare minimum library, which relies on sauron

## Features
- [ ] Smart edit blockmode
    - When typing a key and the next characters next to it is far, say more than 2 space the character is typed in replace mode
        instead of insert mode
    - [ ] Pressing enter should indent, instead of just moving down
- [ ] Allow the editor to render different syntax highlighting scheme to a set of lines
    - Use would be markdown text with code fence in the content
- [ ] Make the keypresses be translated into Commands, so we can use remap such as vi, kakune, etc.

## Maintenance
- [ ] Put the css of each of the component to their own module
    - line, range, cell
- ~~[ ] Make Line, Range and Cell implement View~~
     - This is not possible since these struct has a custom view function which neeeds access to multiple arguments
- [ ] Render only the lines that are visible in the viewport
    - [ ] Determine the number of lines that could fit in the viewport.
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
- [ ] Simplify the text buffer, just a `Vec<Vec<char>>`, no paging, no ranges, no char.

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
- [ ] The long svgbob example is taking a long time to update.
    - 180ms when syntax highlighting is enabled
        - scrolling is 20ms
        - building the view: 50ms
        - patching : 40ms
    - 80ms when syntax highlighting is disabled
- [X] When `replace_char` is called for the next page, it applies to the first page instead
- [ ] replace addition and subtraction operation with saturating_add and saturating_sub.
- [ ] If the top level view of a Program changes, then the original root_node is not set, which causes
    - [ ] Add a test for replacing the top-level root-node and confirm it is changed

## Tests
- [ ] Add test for undo and redo (editor)
