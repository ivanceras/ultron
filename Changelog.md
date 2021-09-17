# Changelog

## Unreleased
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
