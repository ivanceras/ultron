# Ultron

Ultron is a web based monospace text-editor with syntax highlighting, completely written in rust.

![Screenshot](https://raw.githubusercontent.com/ivanceras/ultron/master/screenshot/ultron.png)

## Feature
- Real monospace GUI with support for multi-width characters such as CJK and unicode box drawing.
- Fast, typing latency at ~15ms and cursor move at ~10ms.
- Block mode
    - Allows you to do a rectangular selection
- Virtual edit
    - Allows you to type in anywhere on the editor, even on areas where there is no line


## Syntax-highlighter for static site generator
Ultron comes with `ultron-ssg` crate which can be used for syntax highlighting for a static site generator.

```rust

use ultron_ssg;

fn main() {
    let content = r#"
        fn main(){
            println!("hello from ultron-ssg");
        }
    "#
    let html =
        ultron_ssg::render_to_string(content, "rust", Some("gruvbox-dark"));
    std::fs::create_dir_all("out").expect("must create dir");
    std::fs::write("out/hello.html", html).expect("must write to file");
}
```


## Use-case
I wrote this code editor for my very specific usecase:
- real monospace on GUI editors for ascii diagrams with support for multi-width characters such that it aligns
    with other characters on other lines with respect to their character width.

GUI editors don't handle monospace font quite well for CJK characters or any unicode characters
that have are more than 1 character wide.

Terminal have no problem displaying them.
Fonts in GUI seems to adjust characters on how closely they are lined up together.
That is good for reading and all, but not for Ascii diagrams.

The solution would be to wrap each characters with a `<div>` to force them to be in one cell.
Wide characters will be using `<div class"wide_{n}">` where `n` is the unicode_width.
The style for this char will then be set with a multiplier to the normal width.



## Build and run the editor

```sh
git clone https://github.com/ivanceras/ultron.git

cd ultron
./serve.sh

```
Then, navigate to http://localhost:4002


## Demo

[link](https://ivanceras.github.io/ultron)


#### Patreon link
 [![Become a patron](https://c5.patreon.com/external/logo/become_a_patron_button.png)](https://www.patreon.com/ivanceras)
