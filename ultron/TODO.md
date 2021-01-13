# TODO
- [X] Add blinking cursor
    - [X] I-type
    - [X] block type
- [X] Character typing
- [ ] Cut, Copy and Paste
    - [X] Copy
    - [X] Cut, Copy to clipboard then remove the selected region
    - [X] Paste
        - Use the hacks where a focused textarea is created and the pasted text will
            retrived from there.
            https://stackoverflow.com/questions/2787669/get-html-from-clipboard-in-javascript
    - [ ] Replace, Paste to a selected region
           - The selected region is removed and the clipboard content is inserted
    - [ ] Increase the selection when shift key is pressed
- [X] Move the cursor to the user clicked location
- [X] Redo/undo
    - [X] Undo/Redo on typing
    - [X]  Undo/redo on cut/copy/paste
- [-] Syntax highlighting using syntect
    - Not applicable to svgbob pro
- [ ] multi-layer buffer, so you can drag objects around
- [ ] Optimize rendering of svgbob using debounce and throttle technique
- [-] Use `ex` units in svgbob text fonts
    - [-] Issue: svgbob units are in px
        - [ ] you can actually set the unit in svg: e.g.  `dx="10ex"`
        - Issue: paths have no units
- [ ] Draggable separator
- [X] selection
    - select parts of the text buffer
- [ ] Scroll the view to when the cursor is not visible on the screen
- [ ] Drag and drop shapes
- [ ] Use theming from the futuristic-ui in sauron

# Optimization (for sauron, needed for code-editor)
- [X] Add a specialized `skip(bool)` for html nodes
    - skip(true) means it will not be evaluated for diffing
    - skip(false) or no skip attribute will be evaluated for diffing
    - this will be useful for lines that are not touched or changed
- [ ] `find_nodes` to patch is O(n) complexity
    - Add optimization here: `index(bool)` will create a reference
     to this node and when retrieved from `find_nodes` it would be faster
    - [ ] Alternatively, we create a reference to the actual
     created node for all the virtual Node.
