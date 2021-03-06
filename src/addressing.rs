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

use crate::{envelope::Envelope, helper::element_builder::ElementBuilder};

/// Implementation of [WS-Addressing](https://www.w3.org/Submission/2004/SUBM-ws-addressing-20040810/)
///
/// # Status
/// Incomplete
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Addressing {
    pub action: Option<String>,
    pub message_id: Option<String>,
    pub to: Option<String>,
}

impl Addressing {
    pub fn parse(envelope: &Envelope) -> Self {
        let mut builder = AddressingBuilder::new();
        for header in &envelope.header {
            if let xmltree::XMLNode::Element(element) = header {
                if element.namespace
                    == Some("http://schemas.xmlsoap.org/ws/2004/08/addressing".to_string())
                {
                    if element.name == "Action" {
                        builder = builder.action(element.get_text().unwrap())
                    }
                    if element.name == "MessageID" {
                        builder = builder.message_id(element.get_text().unwrap())
                    }
                    if element.name == "To" {
                        builder = builder.to(element.get_text().unwrap())
                    }
                }
            }
        }
        builder.into()
    }

    pub fn write(&self, envelope: &mut Envelope) {
        if let Some(action) = &self.action {
            let element = ElementBuilder::new("wsa:Action")
                .ns("wsa", "http://schemas.xmlsoap.org/ws/2004/08/addressing")
                .text(action)
                .into();
            envelope.header.push(xmltree::XMLNode::Element(element))
        }
        if let Some(message_id) = &self.message_id {
            let element = ElementBuilder::new("wsa:MessageID")
                .ns("wsa", "http://schemas.xmlsoap.org/ws/2004/08/addressing")
                .text(message_id)
                .into();
            envelope.header.push(xmltree::XMLNode::Element(element))
        }
        if let Some(to) = &self.to {
            let element = ElementBuilder::new("wsa:To")
                .ns("wsa", "http://schemas.xmlsoap.org/ws/2004/08/addressing")
                .text(to)
                .into();
            envelope.header.push(xmltree::XMLNode::Element(element))
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AddressingBuilder {
    addressing: Addressing,
}

impl AddressingBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn action<S>(mut self, text: S) -> Self
    where
        S: Into<String>,
    {
        self.addressing.action = Some(text.into());
        self
    }

    pub fn message_id<S>(mut self, text: S) -> Self
    where
        S: Into<String>,
    {
        self.addressing.message_id = Some(text.into());
        self
    }

    pub fn generate_random_message_id(self) -> Self {
        use uuid::Uuid;
        let uuid = Uuid::new_v4();
        let mut buffer = Uuid::encode_buffer();
        let text = uuid.to_urn().encode_lower(&mut buffer);
        self.message_id(text)
    }

    pub fn to<S>(mut self, text: S) -> Self
    where
        S: Into<String>,
    {
        self.addressing.to = Some(text.into());
        self
    }
}

impl<'a> From<AddressingBuilder> for Addressing {
    #[inline]
    fn from(b: AddressingBuilder) -> Addressing {
        b.addressing
    }
}

#[cfg(test)]
mod tests {
    use crate::addressing::{Addressing, AddressingBuilder};

    #[test]
    fn writer_addressing_printer() {
        use crate::envelope::EmitterConfig;
        use crate::envelope::Envelope;
        use crate::helper::string_buffer::StringBuffer;

        let mut output = StringBuffer::new();

        let config = EmitterConfig::new()
            .perform_indent(true)
            .write_document_declaration(false);

        let mut envelope = Envelope::empty();

        let addressing: Addressing = AddressingBuilder::new()
            .action("http://schemas.xmlsoap.org/ws/2005/04/discovery/Hello")
            .message_id("urn:uuid:94ff5a40-6d87-11b2-8da8-84ba3bbfd024")
            .to("urn:schemas-xmlsoap-org:ws:2005:04:discovery")
            .into();

        addressing.write(&mut envelope);

        envelope.write_with_config(&mut output, config).unwrap();

        // Example 1 from https://www.w3.org/TR/soap12-part1/
        let expected_result = r#"<env:Envelope xmlns:env="http://www.w3.org/2003/05/soap-envelope">
  <env:Header>
    <wsa:Action xmlns:wsa="http://schemas.xmlsoap.org/ws/2004/08/addressing">http://schemas.xmlsoap.org/ws/2005/04/discovery/Hello</wsa:Action>
    <wsa:MessageID xmlns:wsa="http://schemas.xmlsoap.org/ws/2004/08/addressing">urn:uuid:94ff5a40-6d87-11b2-8da8-84ba3bbfd024</wsa:MessageID>
    <wsa:To xmlns:wsa="http://schemas.xmlsoap.org/ws/2004/08/addressing">urn:schemas-xmlsoap-org:ws:2005:04:discovery</wsa:To>
  </env:Header>
  <env:Body />
</env:Envelope>"#;

        assert_eq!(expected_result, output.to_string());
    }

    #[test]
    fn builder_random_message_id() {
        let builder = AddressingBuilder::new().generate_random_message_id();
        let message_id = builder.addressing.message_id.unwrap();
        assert!(message_id.starts_with("urn:uuid:"));
    }

    #[test]
    fn parser_addressing_printer() {
        use crate::envelope::Envelope;

        let input = r#"<env:Envelope xmlns:env="http://www.w3.org/2003/05/soap-envelope">
    <env:Header>
        <wsa:Action xmlns:wsa="http://schemas.xmlsoap.org/ws/2004/08/addressing">http://schemas.xmlsoap.org/ws/2005/04/discovery/Hello</wsa:Action>
        <wsa:MessageID xmlns:wsa="http://schemas.xmlsoap.org/ws/2004/08/addressing">urn:uuid:94ff5a40-6d87-11b2-8da8-84ba3bbfd024</wsa:MessageID>
        <wsa:To xmlns:wsa="http://schemas.xmlsoap.org/ws/2004/08/addressing">urn:schemas-xmlsoap-org:ws:2005:04:discovery</wsa:To>
    </env:Header>
    <env:Body />
</env:Envelope>"#;

        let envelope = Envelope::parse(input.as_bytes()).unwrap();

        let addressing = Addressing::parse(&envelope);

        assert_eq!(
            addressing.action,
            Some("http://schemas.xmlsoap.org/ws/2005/04/discovery/Hello".to_string())
        );
        assert_eq!(
            addressing.message_id,
            Some("urn:uuid:94ff5a40-6d87-11b2-8da8-84ba3bbfd024".to_string())
        );
        assert_eq!(
            addressing.to,
            Some("urn:schemas-xmlsoap-org:ws:2005:04:discovery".to_string())
        );
    }
}
