/* Copyright (C) 2021 Casper Meijn <casper@meijn.net>
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use xmltree::Element;

//TODO: Upstream this struct to xmltree
/// A builder for Element
pub struct ElementBuilder {
    element: Element,
}

impl ElementBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            element: Element::new(name),
        }
    }

    /// Sets an attribute value of this element to the given string.
    ///
    /// This method can be used to add attributes to the element. Name is a plain local
    /// name. Duplicate values are overridden.
    ///
    /// The writer checks that you don't specify reserved prefix names, for example `xmlns`.
    #[inline]
    pub fn attr<S1, S2>(mut self, name: S1, value: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        self.element.attributes.insert(name.into(), value.into());
        self
    }

    /// Adds a namespace to the current namespace context.
    ///
    /// If no namespace URI was bound to the provided prefix at this point of the document,
    /// then the mapping from the prefix to the provided namespace URI will be written as
    /// a part of this element attribute set.
    ///
    /// If the same namespace URI was bound to the provided prefix at this point of the document,
    /// then no namespace attributes will be emitted.
    ///
    /// If some other namespace URI was bound to the provided prefix at this point of the document,
    /// then another binding will be added as a part of this element attribute set, shadowing
    /// the outer binding.
    pub fn ns<S1, S2>(mut self, prefix: S1, uri: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        if let Some(namespaces) = self.element.namespaces.as_mut() {
            namespaces.put(prefix, uri);
        } else {
            let mut namespaces = xmltree::Namespace::empty();
            namespaces.put(prefix, uri);
            self.element.namespaces = Some(namespaces);
        }
        self
    }

    /// Adds a XMLNode::Text as a child of the Element
    pub fn text<S>(mut self, text: S) -> Self
    where
        S: Into<String>,
    {
        self.element
            .children
            .push(xmltree::XMLNode::Text(text.into()));
        self
    }

    /// Adds another Element as a child of this Element
    pub fn child<E>(mut self, element: E) -> Self
    where
        E: Into<Element>,
    {
        self.element
            .children
            .push(xmltree::XMLNode::Element(element.into()));
        self
    }
}

impl<'a> From<ElementBuilder> for Element {
    #[inline]
    fn from(b: ElementBuilder) -> Element {
        b.element
    }
}
