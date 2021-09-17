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

use std::io::Write;

/// Buffer from writing bytes to. The buffer can then be converted to a String
///
/// # Example
/// ```
/// use soap::helper::string_buffer::StringBuffer;
/// use std::io::Write;
/// let mut buffer = StringBuffer::new();
/// buffer.write(b"Hello ");
/// buffer.write(b"world!");
/// let text: String = buffer.into();
/// assert_eq!(String::from("Hello world!"), text);
/// ```
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct StringBuffer {
    buf: Vec<u8>,
}

impl StringBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn to_string(self) -> String {
        String::from_utf8(self.buf).unwrap()
    }
}

impl Write for StringBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl From<StringBuffer> for String {
    fn from(buffer: StringBuffer) -> Self {
        buffer.to_string()
    }
}
