# TODO

## Functionality
- [ ] Cut, Copy and Paste
        - Use the hacks where a focused textarea is created and the pasted text will
            retrived from there.
            https://stackoverflow.com/questions/2787669/get-html-from-clipboard-in-javascript
    - [ ] Replace, Paste to a selected region
           - The selected region is removed and the clipboard content is inserted
    - [ ] Increase the selection when shift key is pressed
- [X] Move the cursor to the user clicked location
- [ ] Redo/undo
    - [ ] Undo/Redo on typing
    - [ ] Undo/redo on cut/copy/paste
- [ ] multi-layer buffer, so you can drag objects around
- [X] Draggable separator
- [X] selection
    - select parts of the text buffer
- [ ] Scroll the view to when the cursor is not visible on the screen
- [ ] Centralize the commands into actions first, so we can send an external messages
    and hook it into history for undor/redo functionality

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
