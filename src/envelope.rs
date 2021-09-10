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

use std::{
    fmt,
    io::{Read, Write},
};

pub use xml::writer::Error as WriteError;
pub use xml::EmitterConfig;
use xmltree::Element;

/// Container for an SOAP Envelope
pub struct Envelope {
    pub header: Vec<xmltree::XMLNode>,
    pub body: Vec<xmltree::XMLNode>,
}

impl Envelope {
    /// Create an empty Envelope
    pub fn empty() -> Self {
        Self {
            header: vec![],
            body: vec![],
        }
    }

    /// Parses xml data into an Envelope
    pub fn parse<R: Read>(r: R) -> Result<Envelope, ParseError> {
        let xmlnode = xmltree::Element::parse(r).map_err(|e| ParseError::MalformedXml(e))?;

        if xmlnode.name != "Envelope" {
            return Err(ParseError::InvalidEnvelopeName);
        }

        let mut envelope = Self::empty();

        if let Some(child) = xmlnode.get_child("Header") {
            envelope.header = child.children.clone();
        }
        if let Some(child) = xmlnode.get_child("Body") {
            envelope.body = child.children.clone();
        } else {
            return Err(ParseError::MissingBody);
        }

        Ok(envelope)
    }

    /// Writes out this Envelope as the root element in an new XML document
    pub fn write<W: Write>(&self, w: W) -> Result<(), WriteError> {
        self.write_with_config(w, EmitterConfig::new())
    }

    /// Writes out this Envelope as the root element in a new XML document using the provided configuration
    pub fn write_with_config<W: Write>(
        &self,
        w: W,
        config: EmitterConfig,
    ) -> Result<(), WriteError> {
        use crate::helper::element_builder::ElementBuilder;

        let envelope: Element = {
            let mut builder = ElementBuilder::new("env:Envelope")
                .ns("env", "http://www.w3.org/2003/05/soap-envelope");

            let mut header = xmltree::Element::new("env:Header");
            header.children = self.header.clone();
            builder = builder.child(header);

            let mut body = xmltree::Element::new("env:Body");
            body.children = self.body.clone();
            builder = builder.child(body);

            builder.into()
        };

        envelope.write_with_config(w, config)
    }
}

/// Errors that can occur parsing XML
#[derive(Debug)]
pub enum ParseError {
    /// The XML is invalid
    MalformedXml(xmltree::ParseError),
    /// Root tag of the parsed data is not `Envelope`
    InvalidEnvelopeName,
    /// No `Body` child was found in the `Envelope`
    MissingBody,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::MalformedXml(ref e) => write!(f, "Malformed XML. {}", e),
            ParseError::InvalidEnvelopeName => write!(f, "Invalid envelope name"),
            ParseError::MissingBody => write!(f, "Missing body"),
        }
    }
}

impl std::error::Error for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::MalformedXml(..) => "Malformed XML",
            ParseError::InvalidEnvelopeName => "Invalid envelope name",
            ParseError::MissingBody => "Missing body",
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            ParseError::MalformedXml(ref e) => Some(e),
            ParseError::InvalidEnvelopeName => None,
            ParseError::MissingBody => None,
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn reader_soap12_part1_example1() {
        use crate::envelope::Envelope;

        // Example 1 from https://www.w3.org/TR/soap12-part1/
        let input = r#"<env:Envelope xmlns:env="http://www.w3.org/2003/05/soap-envelope">
<env:Header>
<n:alertcontrol xmlns:n="http://example.org/alertcontrol">
  <n:priority>1</n:priority>
  <n:expires>2001-06-22T14:00:00-05:00</n:expires>
</n:alertcontrol>
</env:Header>
<env:Body>
<m:alert xmlns:m="http://example.org/alert">
  <m:msg>Pick up Mary at school at 2pm</m:msg>
</m:alert>
</env:Body>
</env:Envelope>"#;

        let envelope = Envelope::parse(input.as_bytes()).unwrap();
        assert_eq!(
            "alertcontrol",
            envelope.header.first().unwrap().as_element().unwrap().name
        );
        assert_eq!(
            "http://example.org/alertcontrol",
            envelope
                .header
                .first()
                .unwrap()
                .as_element()
                .unwrap()
                .namespace
                .as_ref()
                .unwrap()
        );
        assert_eq!(
            "alert",
            envelope.body.first().unwrap().as_element().unwrap().name
        );
        assert_eq!(
            "http://example.org/alert",
            envelope
                .body
                .first()
                .unwrap()
                .as_element()
                .unwrap()
                .namespace
                .as_ref()
                .unwrap()
        );
    }

    fn build_alertcontrol_header() -> xmltree::Element {
        use crate::helper::element_builder::ElementBuilder;

        let priority = ElementBuilder::new("n:priority").text("1");

        let expires = ElementBuilder::new("n:expires").text("2001-06-22T14:00:00-05:00");

        ElementBuilder::new("n:alertcontrol")
            .ns("n", "http://example.org/alertcontrol")
            .child(priority)
            .child(expires)
            .into()
    }

    fn build_alert_body() -> xmltree::Element {
        use crate::helper::element_builder::ElementBuilder;

        let msg = ElementBuilder::new("m:msg").text("Pick up Mary at school at 2pm");

        ElementBuilder::new("m:alert")
            .ns("m", "http://example.org/alert")
            .child(msg)
            .into()
    }

    #[test]
    fn writer_soap12_part1_example1() {
        use crate::envelope::EmitterConfig;
        use crate::envelope::Envelope;
        use crate::helper::string_buffer::StringBuffer;

        let mut output = StringBuffer::new();

        let config = EmitterConfig::new()
            .perform_indent(true)
            .write_document_declaration(false);

        let envelope = Envelope {
            header: vec![xmltree::XMLNode::Element(build_alertcontrol_header())],
            body: vec![xmltree::XMLNode::Element(build_alert_body())],
        };

        envelope.write_with_config(&mut output, config).unwrap();

        // Example 1 from https://www.w3.org/TR/soap12-part1/
        let expected_result = r#"<env:Envelope xmlns:env="http://www.w3.org/2003/05/soap-envelope">
  <env:Header>
    <n:alertcontrol xmlns:n="http://example.org/alertcontrol">
      <n:priority>1</n:priority>
      <n:expires>2001-06-22T14:00:00-05:00</n:expires>
    </n:alertcontrol>
  </env:Header>
  <env:Body>
    <m:alert xmlns:m="http://example.org/alert">
      <m:msg>Pick up Mary at school at 2pm</m:msg>
    </m:alert>
  </env:Body>
</env:Envelope>"#;

        assert_eq!(expected_result, output.to_string());
    }
}
