use crate::sections::PsdCursor;

pub(crate) struct RLECompressed<'a> {
    cursor: PsdCursor<'a>,
    repeat: usize,
    literal: Option<u8>,
}

impl<'a> RLECompressed<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> RLECompressed<'a> {
        RLECompressed {
            cursor: PsdCursor::new(bytes),
            literal: None,
            repeat: 0,
        }
    }
}

impl<'a> Iterator for RLECompressed<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.repeat > 0 {
            self.repeat -= 1;
            return match self.literal {
                Some(_) => self.literal,
                None => Some(self.cursor.read_u8()),
            };
        }

        if self.cursor.position() >= self.cursor.get_ref().len() as u64 {
            return None;
        }

        if self.repeat == 0 {
            let header = self.cursor.read_i8() as i16;
            if header == -128 || self.cursor.position() == self.cursor.get_ref().len() as u64 {
                return self.next();
            }

            if header >= 0 {
                self.literal = None;
                self.repeat = 1 + header as usize
            } else {
                self.literal = Some(self.cursor.read_u8());
                self.repeat = (1 - header) as usize
            }
        }

        self.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let empty = vec![];
        assert_eq!(RLECompressed::new(&empty).collect::<Vec<u8>>(), empty);
    }

    #[test]
    fn test_literal() {
        let value = vec![0, 1, 0, 2, 0, 3, 0, 4];
        assert_eq!(
            RLECompressed::new(&value).collect::<Vec<u8>>(),
            vec![1, 2, 3, 4]
        );
    }

    #[test]
    fn test_repeat() {
        let value = vec![253, 1];
        assert_eq!(
            RLECompressed::new(&value).collect::<Vec<u8>>(),
            vec![1, 1, 1, 1]
        );
    }
}
