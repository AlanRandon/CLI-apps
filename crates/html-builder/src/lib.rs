use html_escape::encode_safe as escape;
use itertools::Itertools;
use prelude::*;
use std::{collections::HashMap, fmt::Display};

pub fn document<H, I>(
    head: impl IntoIterator<Item = H>,
    body: impl IntoIterator<Item = I>,
) -> String
where
    Node: From<H> + From<I>,
{
    format!(
        "<!DOCTYPE html>{}",
        html()
            .attr("lang", "en")
            .child(
                prelude::head()
                    .child(meta().attr("charset", "utf-8"))
                    .child(
                        meta()
                            .attr("name", "viewport")
                            .attr("content", "width=device-width, initial-scale=1"),
                    )
                    .children(head.into_iter())
            )
            .child(prelude::body().children(body.into_iter()))
    )
}

pub fn text(s: impl Display) -> Node {
    Node::Text(Text(s.to_string()))
}

pub fn raw_text(s: impl Display) -> Node {
    Node::RawHtml(s.to_string())
}

#[derive(Debug, Clone)]
pub enum Node {
    Element(Element),
    Text(Text),
    RawHtml(String),
    VoidElement(VoidElement),
}

impl From<Element> for Node {
    fn from(element: Element) -> Self {
        Node::Element(element)
    }
}

impl From<VoidElement> for Node {
    fn from(element: VoidElement) -> Self {
        Node::VoidElement(element)
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(text) => text.fmt(f),
            Self::Element(element) => element.fmt(f),
            Self::RawHtml(html) => html.fmt(f),
            Self::VoidElement(element) => element.fmt(f),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Text(pub String);

impl Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        escape(&self.0).fmt(f)
    }
}

#[derive(Debug, Clone)]
pub struct Element {
    name: String,
    attributes: Attributes,
    children: Vec<Node>,
}

impl Element {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            attributes: Attributes::default(),
            children: Vec::new(),
        }
    }

    pub fn attr(mut self, key: impl Display, value: impl Display) -> Self {
        self.attributes.0.insert(key.to_string(), value.to_string());
        self
    }

    pub fn child(mut self, child: impl Into<Node>) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn children(mut self, children: impl Iterator<Item = impl Into<Node>>) -> Self {
        self.children.extend(children.map(Into::into));
        self
    }

    pub fn opt_child(self, child: Option<impl Into<Node>>) -> Self {
        self.children(child.into_iter())
    }

    pub fn text(self, text: impl Display) -> Self {
        self.child(Node::Text(Text(text.to_string())))
    }
}

impl Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<{0}{1}>{2}</{0}>",
            self.name,
            self.attributes,
            self.children.iter().map(ToString::to_string).join("")
        )
    }
}

#[derive(Debug, Clone)]
pub struct VoidElement {
    name: String,
    attributes: Attributes,
}

impl VoidElement {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            attributes: Attributes::default(),
        }
    }

    pub fn attr(mut self, key: impl Display, value: impl Display) -> Self {
        self.attributes.0.insert(key.to_string(), value.to_string());
        self
    }
}

impl Display for VoidElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}{}/>", self.name, self.attributes)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Attributes(HashMap<String, String>);

impl Display for Attributes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            return Ok(());
        }

        write!(
            f,
            " {}",
            self.0
                .iter()
                .map(|(key, value)| (escape(key), escape(value)))
                .map(|(key, value)| format!(r#"{key}="{value}""#))
                .join(" ")
        )
    }
}

pub mod prelude {
    pub use super::html::{self, *};
}

pub mod html {
    pub use super::{document, text, Attributes, Element, Node, Text, VoidElement};
    use std::fmt::Display;

    macro_rules! repeat {
            ($macro:ident, $($names:ident),*) => {
                $($macro!($names);)*
            };
        }

    macro_rules! element {
        ($name:ident) => {
            #[allow(dead_code)]
            pub fn $name() -> Element {
                Element::new(stringify!($name))
            }
        };
    }

    repeat!(
        element, a, abbr, address, article, aside, audio, b, bdi, bdo, blockquote, body, button,
        canvas, caption, cite, code, colgroup, data, datalist, dd, del, details, dfn, dialog, div,
        dl, dt, em, fieldset, figcaption, figure, footer, form, h1, h2, h3, h4, h5, h6, head,
        header, hgroup, html, i, iframe, ins, kbd, label, legend, li, main, map, mark, menu, meter,
        nav, noscript, object, ol, optgroup, option, output, p, picture, portal, pre, progress, q,
        rp, rt, ruby, s, samp, script, section, select, slot, small, span, strong, style, sub,
        summary, sup, table, tbody, td, template, textarea, tfoot, th, thead, time, title, tr, u,
        ul, var, video
    );

    macro_rules! void_element {
        ($name:ident) => {
            #[allow(dead_code)]
            pub fn $name() -> VoidElement {
                VoidElement::new(stringify!($name))
            }
        };
    }

    repeat!(
        void_element,
        area,
        base,
        br,
        col,
        embed,
        hr,
        img,
        input,
        link,
        meta,
        param,
        source,
        track,
        wbr
    );

    macro_rules! attr {
        ($name:ident) => {
            impl Element {
                #[allow(dead_code)]
                pub fn $name(self, value: impl Display) -> Self {
                    self.attr(stringify!($name), value)
                }
            }

            impl VoidElement {
                #[allow(dead_code)]
                pub fn $name(self, value: impl Display) -> Self {
                    self.attr(stringify!($name), value)
                }
            }
        };
    }

    repeat!(attr, class, href, id);
}
