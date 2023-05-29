use std::{
    fmt::{Display, Formatter},
    str::from_utf8_unchecked,
};

use tracing::trace;

/// Implementation of a circular buffer for string.
/// Unlike a regular vector, this one never extends the underlying buffer.
/// Instead, it wraps around to the beginning of the buffer.
/// The strings inserted are always separated by a newline. If there string
/// already ends in a newline, it is not added.

#[derive(Debug)]
pub struct CircularString {
    buffer: Vec<u8>,
    len1:   usize,
    gap:    usize,
    len2:   usize,
}

impl CircularString {
    pub fn with_capacity(capacity: usize) -> CircularString {
        CircularString {
            buffer: vec![0; capacity],
            len1:   0,
            gap:    capacity,
            len2:   0,
        }
    }

    pub fn len(&self) -> usize {
        self.len1 + self.len2
    }

    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// push a str into the buffer
    /// automatically adds an EOL if the string doesn't have one
    pub fn push(&mut self, s: &str) {
        trace!("pushing: {}", s);
        let add_eol = !s.ends_with('\n');
        let bytes = s.as_bytes();
        let len = bytes.len() + if add_eol { 1 } else { 0 };
        if len > self.capacity() {
            panic!("buffer overflow, string too big to fit in buffer");
        }
        let l1 = self.len1;
        let l2 = self.len2;
        let gap = self.gap;

        macro_rules! copy_buffer {
            ($self:ident, $bytes:ident, $offset:expr) => {
                self.buffer[$offset..$offset + $bytes.len()].copy_from_slice($bytes);
                if bytes.last() != Some(&b'\n') {
                    self.buffer[$offset + $bytes.len()] = b'\n';
                }
            };
        }

        // |<---------------- capacity ---------------------->|
        // |<-- len1 -->|<-- gap -->|<-- len2 -->|<--      -->|
        // There is a total of seven cases to consider. However, the cases when
        // we rotate can be reduced to two if we decide to discard the rest of len2
        match () {
            _ if len <= gap => {
                trace!("copy the bytes into the buffer at the end of the first part, gap is big enough");
                copy_buffer!(self, bytes, l1);
                self.len1 += len;
                self.gap -= len;
            }
            _ if len <= gap + l2 => {
                trace!("copy starting at the beginning of the gap, overwrite into l2");
                let from_l2 = find_boundary(&self.buffer[l1 + gap..l1 + gap + l2], len - gap);
                copy_buffer!(self, bytes, l1);
                self.len1 += len;
                self.gap = gap + from_l2 - len;
                self.len2 = l2 - from_l2;
            }
            _ if len <= self.capacity() - l1 => {
                trace!("copy starting from the gap, completely overwrite l2");
                copy_buffer!(self, bytes, l1);
                self.len1 += len;
                self.gap = self.capacity() - self.len1;
                self.len2 = 0;
            }
            // from here on, we need to rotate the buffer
            _ if len <= l1 => {
                trace!("copy at the start of the buffer, we split the l1 into l1, gap and l2 and the rest becomes the leftover");
                let min_size = find_boundary(&self.buffer[..l1], len);
                copy_buffer!(self, bytes, 0);
                self.len1 = len;
                self.gap = min_size - len;
                self.len2 = l1 - min_size;
            }
            _ => {
                trace!("copy at start of buffer, new string is the whole content");
                copy_buffer!(self, bytes, 0);
                self.len1 = len;
                self.gap = self.capacity() - len;
                self.len2 = 0;
            }
        }
        trace!("new value: {:#?}", self);
    }
}

impl<'a> IntoIterator for &'a CircularString {
    type Item = &'a str;
    type IntoIter = std::iter::Chain<std::str::Lines<'a>, std::str::Lines<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        unsafe {
            from_utf8_unchecked(
                &self.buffer[self.len1 + self.gap..self.len1 + self.gap + self.len2],
            )
        }
        .lines()
        .chain(unsafe { from_utf8_unchecked(&self.buffer[..self.len1]) }.lines())
    }
}

/// find where the gap is
fn find_boundary(buffer: &[u8], size: usize) -> usize {
    let s = unsafe { from_utf8_unchecked(&buffer) };
    for (i, c) in s.char_indices() {
        if c == '\n' {
            if i + 1 >= size {
                return i + 1;
            }
        }
    }
    buffer.len()
}

#[cfg(test)]
mod boundary_tests {
    use tracing_test::traced_test;

    use super::*;

    #[test]
    #[traced_test]
    fn find_aligned() {
        let buffer = b"hello\nworld\n";
        let size = 12;
        let boundary = find_boundary(buffer, size);
        assert_eq!(boundary, 12);
    }

    #[test]
    #[traced_test]
    fn find_unaligned() {
        let buffer = b"hello\nworld\n";
        let size = 3;
        let boundary = find_boundary(buffer, size);
        assert_eq!(boundary, 6);
    }

    #[test]
    #[traced_test]
    fn find_non_ascii() {
        let buffer = "hellö\nworld\n";
        let size = 7;
        let boundary = find_boundary(buffer.as_bytes(), size);
        assert_eq!(boundary, 7);
    }
}

impl Display for CircularString {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let s1 = unsafe { from_utf8_unchecked(&self.buffer[..self.len1]) };
        let s2 = unsafe {
            from_utf8_unchecked(
                &self.buffer[self.len1 + self.gap..self.len1 + self.gap + self.len2],
            )
        };
        trace!(
            "s2: {}, len1: {}, gap: {}, len2: {}, start: {}, end: {}",
            s2,
            self.len1,
            self.gap,
            self.len2,
            self.len1 + self.gap,
            self.len1 + self.gap + self.len2
        );
        write!(f, "{}{}", s2, s1)
    }
}

#[cfg(test)]
mod push_tests {
    use tracing_test::traced_test;

    use super::*;

    #[test]
    #[traced_test]
    fn test_with_no_rotation() {
        let mut cs = CircularString::with_capacity(12);
        cs.push("hello");
        assert_eq!(cs.to_string(), "hello\n");
        cs.push("world");
        assert_eq!(cs.to_string(), "hello\nworld\n");
        assert_eq!((cs.len1, cs.gap, cs.len2), (12, 0, 0));
    }

    #[test]
    #[traced_test]
    fn test_circular_string() {
        let mut cs = CircularString::with_capacity(12);
        cs.push("hello");
        cs.push("world");
        // new the buffer is full, next push will rotate
        cs.push("aha");
        assert_eq!(cs.to_string(), "world\naha\n");
        assert_eq!((cs.len1, cs.gap, cs.len2), (4, 2, 6));
    }

    #[test]
    #[traced_test]
    fn test_writing_in_the_gap() {
        let mut cs = CircularString::with_capacity(12);
        cs.push("hello");
        cs.push("world");
        cs.push("aha");
        cs.push("!");
        assert_eq!(cs.to_string(), "world\naha\n!\n");
        assert_eq!((cs.len1, cs.gap, cs.len2), (6, 0, 6));
    }

    #[test]
    #[traced_test]
    fn test_writing_over_all_l2() {
        let mut cs = CircularString::with_capacity(12);
        cs.push("hello");
        cs.push("world");
        cs.push("aha");
        cs.push("!");
        cs.push("foo");
        assert_eq!(cs.to_string(), "aha\n!\nfoo\n");
        assert_eq!((cs.len1, cs.gap, cs.len2), (10, 2, 0));
    }

    #[test]
    #[traced_test]
    fn test_discarding_leftover_l2() {
        let mut cs = CircularString::with_capacity(13);
        cs.push("hello");
        cs.push("world");
        cs.push("aha");
        assert_eq!((cs.len1, cs.gap, cs.len2), (4, 2, 6));
        cs.push("12345678");
        assert_eq!(cs.to_string(), "aha\n12345678\n");
        assert_eq!((cs.len1, cs.gap, cs.len2), (13, 0, 0));
    }
}

#[cfg(test)]
mod iterator_tests {
    use tracing_test::traced_test;

    use super::*;

    #[test]
    #[traced_test]
    fn test_iter() {
        let mut cs = CircularString::with_capacity(12);
        cs.push("hello");
        cs.push("world");
        cs.push("aha");
        cs.push("!");
        cs.push("foo");
        let mut iter = cs.into_iter();
        assert_eq!(iter.next(), Some("aha"));
        assert_eq!(iter.next(), Some("!"));
        assert_eq!(iter.next(), Some("foo"));
        assert_eq!(iter.next(), None);
    }
}
