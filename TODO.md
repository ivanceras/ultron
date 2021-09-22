# TODO

## Functionality
- [ ] Cut, Copy and Paste
        - Use the hacks where a focused textarea is created and the pasted text will
            retrived from there.
            https://stackoverflow.com/questions/2787669/get-html-from-clipboard-in-javascript
    - [X] Replace, Paste to a selected region
           - The selected region is removed and the clipboard content is inserted
    - [ ] Increase the selection when shift key is pressed
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
- [ ] Add an api to group action together in Recorded
        ```rust
        pub struct Recorded{
            history: VecDeque<Vec<Action>>,
            undone: VecDeque<Vec<Action>>,
        }
        ```
- [ ] Context Menu (Right click)
    - undo/redo
    - cut/copy/paste

## Features
- [ ] Smart edit blockmode
    - When typing a key and the next characters next to it is far, say more than 2 space the character is typed in replace mode
        instead of insert mode

## Maintenance
- [ ] Put the css of each of the component to their own module
    - line, range, cell
- ~~[ ] Make Line, Range and Cell implement View~~
     - This is not possible since these struct has a custom view function which neeeds access to multiple arguments

## Issues
- [ ] Moving around characters with cell_width of 2
        should not need to pressed the arrow right 2 times to get ot the next character.
        This is because movement is merely adding +1 to x or y position.
        - [ ] Need to make cursor movement line cell aware.
- [ ] When typing very fast, the content in the textarea is accumulated and there is a duplicate on the characters
