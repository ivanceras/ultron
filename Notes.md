# Notes
- The reason why we can not just slap in mouse events on the div for each Cell is that
    the cells can overlap each other. In addition, some other divs maybe covering the cell such as cursors.
    - The best way to do this is calculate the cursor location relative to the top_left of the editor window.
- Caching page_views will result in much faster view time.
