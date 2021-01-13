# Ultron

Ultron is a web based monospaced text-editor completely written in rust.

![Screenshot](https://raw.githubusercontent.com/ivanceras/ultron/master/screenshot/ultron.png)

I wrote this code editor for my very specific usecase:
- real monospace on GUI editors for ascii diagrams

GUI editors don't handle monospace font quite well for CJK characters or any unicode characters
that have are more than 1 character wide.

Terminal have no problem displaying them.
Fonts in GUI seems to adjust characters on how closely they are lined up together.
That is good for reading and all, but not for Ascii diagrams.

The solution would be to wrap each characters with a `<div>` to force them to be in one cell.
Wide characters will be using `<div class"wide_{n}">` where `n` is the unicode_width.
The style for this char will then be set with a multiplier to the normal width.

# Dependencies:
Ropey - to handle the buffer.


Reference:
[Smith](https://github.com/IGI-111/Smith) to see how it is done.
[zee](https://crates.io/crates/zee)

Note:  We can not use xi-editor for this, because this needs to run on the browser.
Xi-core requires opening channels and have a protocol for passing and receiving data from the backend.
Ropey will do that part.

## Build and run the project

```sh

cd ultron-web
./start_optimized.sh

```

## Demo

[link](https://ivanceras.github.io/ultron)


#### Patreon link
 [![Become a patron](https://c5.patreon.com/external/logo/become_a_patron_button.png)](https://www.patreon.com/ivanceras)
