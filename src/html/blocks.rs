//! The elements that decide how a page's text is laid out when it is read as
//! text: the ones a browser never renders as text, the section headings, and
//! the elements that start a new line
//! (https://html.spec.whatwg.org/multipage/rendering.html#hidden-elements,
//! https://html.spec.whatwg.org/multipage/sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements,
//! https://html.spec.whatwg.org/multipage/rendering.html#the-css-user-agent-style-sheet-and-presentational-hints).

/// An element whose content a browser never renders as text: the raw text
/// elements, the inert template, the no-script fallback and embedded SVG.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Hidden {
    Script,
    Style,
    Template,
    Noscript,
    Svg,
}

impl Hidden {
    const ALL: &'static [Hidden] = &[
        Hidden::Script,
        Hidden::Style,
        Hidden::Template,
        Hidden::Noscript,
        Hidden::Svg,
    ];

    fn tag(self) -> &'static str {
        match self {
            Hidden::Script => "script",
            Hidden::Style => "style",
            Hidden::Template => "template",
            Hidden::Noscript => "noscript",
            Hidden::Svg => "svg",
        }
    }

    /// The hidden element a lowercase tag name names; none for an element a
    /// browser renders.
    pub(crate) fn named(tag: &str) -> Option<Hidden> {
        Self::ALL
            .iter()
            .copied()
            .find(|element| element.tag() == tag)
    }
}

/// A section heading, written in the page's text as the Markdown heading of
/// its rank.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Heading {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl Heading {
    const ALL: &'static [Heading] = &[
        Heading::H1,
        Heading::H2,
        Heading::H3,
        Heading::H4,
        Heading::H5,
        Heading::H6,
    ];

    fn tag(self) -> &'static str {
        match self {
            Heading::H1 => "h1",
            Heading::H2 => "h2",
            Heading::H3 => "h3",
            Heading::H4 => "h4",
            Heading::H5 => "h5",
            Heading::H6 => "h6",
        }
    }

    /// The Markdown heading marker of this rank.
    pub(crate) fn marker(self) -> &'static str {
        match self {
            Heading::H1 => "#",
            Heading::H2 => "##",
            Heading::H3 => "###",
            Heading::H4 => "####",
            Heading::H5 => "#####",
            Heading::H6 => "######",
        }
    }

    pub(crate) fn named(tag: &str) -> Option<Heading> {
        Self::ALL
            .iter()
            .copied()
            .find(|heading| heading.tag() == tag)
    }
}

/// An element that starts its content on a new line of the page's text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Block {
    Paragraph,
    TableRow,
    Section,
    Article,
    LineBreak,
    Division,
    Quotation,
}

impl Block {
    const ALL: &'static [Block] = &[
        Block::Paragraph,
        Block::TableRow,
        Block::Section,
        Block::Article,
        Block::LineBreak,
        Block::Division,
        Block::Quotation,
    ];

    fn tag(self) -> &'static str {
        match self {
            Block::Paragraph => "p",
            Block::TableRow => "tr",
            Block::Section => "section",
            Block::Article => "article",
            Block::LineBreak => "br",
            Block::Division => "div",
            Block::Quotation => "blockquote",
        }
    }

    pub(crate) fn named(tag: &str) -> Option<Block> {
        Self::ALL.iter().copied().find(|block| block.tag() == tag)
    }
}
