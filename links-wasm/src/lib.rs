use wasm_bindgen::prelude::*;

use pulldown_cmark::{html, Options, Parser, RenderingOptions};

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/*
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
*/
#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn memo_decrypt(text: &str, password: &str) -> String {
    memo_rust::memo_decrypt(text, password)
}

#[wasm_bindgen]
pub fn memo_encrypt(text: &str, password: &str, nonce: f64) -> String {
    match memo_rust::memo_encrypt(text, password, nonce as u64) {
        Ok(encrypted) => encrypted,
        Err(s) => String::from(s),
    }
}

#[wasm_bindgen]
pub fn transform_markdown(markdown_input: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(markdown_input, options);

    let mut html_output = String::new();
    let mut rendering_options = RenderingOptions::empty();
    rendering_options.insert(RenderingOptions::OPEN_LINK_IN_NEW_TAB);
    html::push_html_ext(&mut html_output, parser, rendering_options);
    html_output
}

/// Lines starting with 3 Japanese dakuten signs are comments.
fn remove_comments(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for line in input.lines() {
        if !line.starts_with("\u{309B}\u{309B}\u{309B}") {
            output.push_str(line);
            output.push('\n');
        }
    }
    output
}

#[wasm_bindgen]
pub fn process_markdown(markdown_input: &str, base_64_limit: usize) -> String {
    if base_64_limit < 1 {
        transform_markdown(&remove_comments(markdown_input))
    } else {
        transform_markdown(&memo_rust::truncate_base64(
            &remove_comments(markdown_input),
            base_64_limit,
        ))
    }
}
