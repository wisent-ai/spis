//! A page read as text: its title, and its `<main>` content when it marks one
//! (https://html.spec.whatwg.org/multipage/grouping-content.html#the-main-element),
//! otherwise all of it, without the elements that render no text, with
//! headings, list items and blocks on lines of their own and preformatted
//! text fenced. The page is the tree `scraper` parses, so a `>` inside a
//! quoted attribute, a comment or a character reference is read as a browser
//! reads it; a hand-written scan cut a tag at its first `>` and leaked the
//! rest of the tag into the text, and never decoded a character reference.

use scraper::node::Node;
use scraper::{ElementRef, Html};

use super::blocks::{Block, Heading, Hidden};

const FENCE: &str = "\n```\n";

struct Reading {
    out: String,
    title: String,
    /// Only text inside `<main>` is kept: the page marks its main content.
    main_only: bool,
    in_main: bool,
    in_pre: bool,
}

impl Reading {
    fn kept(&self) -> bool {
        !self.main_only || self.in_main
    }

    fn text(&mut self, text: &str) {
        if !self.kept() {
            return;
        }
        let squeezed = squeeze(text);
        if squeezed.is_empty() {
            return;
        }
        if !self.in_pre && !self.out.is_empty() && !self.out.ends_with('\n') {
            self.out.push(' ');
        }
        self.out.push_str(&squeezed);
    }

    fn push(&mut self, written: &str) {
        if self.kept() {
            self.out.push_str(written);
        }
    }

    fn children(&mut self, element: ElementRef<'_>) {
        for child in element.children() {
            match child.value() {
                Node::Text(text) => self.text(text),
                Node::Element(_) => {
                    if let Some(inner) = ElementRef::wrap(child) {
                        self.element(inner);
                    }
                }
                _ => {}
            }
        }
    }

    fn element(&mut self, element: ElementRef<'_>) {
        let name = element.value().name();
        if Hidden::named(name).is_some() {
            return;
        }
        if name == "title" {
            self.title.push_str(&squeeze(&element.text().collect::<String>()));
            return;
        }
        if name == "main" {
            let outer = self.in_main;
            self.in_main = true;
            self.children(element);
            self.in_main = outer;
            return;
        }
        if name == "pre" {
            let outer = self.in_pre;
            self.push(FENCE);
            self.in_pre = true;
            self.children(element);
            self.in_pre = outer;
            self.push(FENCE);
            return;
        }
        if !self.in_pre {
            if let Some(heading) = Heading::named(name) {
                self.push(&format!("\n{} ", heading.marker()));
            } else if name == "li" {
                self.push("\n- ");
            } else if Block::named(name).is_some() {
                self.push("\n");
            }
        }
        self.children(element);
    }
}

/// `text` with every run of whitespace written as one space, and none at
/// its start.
fn squeeze(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut space = false;
    for character in text.chars() {
        if character.is_whitespace() {
            if !space && !out.is_empty() {
                out.push(' ');
            }
            space = true;
        } else {
            out.push(character);
            space = false;
        }
    }
    out
}

/// The text and title of the HTML page `html`.
pub fn extract_text(html: &str) -> (String, String) {
    let page = Html::parse_document(html);
    let main_only = page
        .root_element()
        .descendants()
        .any(|node| node.value().as_element().is_some_and(|element| element.name() == "main"));
    let mut reading =
        Reading { out: String::new(), title: String::new(), main_only, in_main: false, in_pre: false };
    reading.element(page.root_element());
    let mut raw = reading.out;
    while raw.contains("\n\n\n") {
        raw = raw.replace("\n\n\n", "\n\n");
    }
    let cleaned = raw.lines().map(str::trim_end).collect::<Vec<_>>().join("\n");
    (cleaned.trim().to_string(), reading.title.trim().to_string())
}
