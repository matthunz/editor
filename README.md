# Engrave
Text editor for desktop and the web with Rust and Dioxus.

```rust
use dioxus::prelude::*;
use engrave::{language, Editor, UseEditor};

fn app(cx: Scope) -> Element {
    let editor = UseEditor::builder()
        .language(language::rust())
        .height(600.)
        .use_editor(cx, || include_str!("../src/editor/use_editor.rs"));

    render!(Editor { editor: editor })
}
```
