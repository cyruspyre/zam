use super::{r#trait::Trait, Parser};

pub type Generic = Vec<(String, Vec<Trait>)>;

impl Parser {
    pub fn gen(&mut self) -> Option<Generic> {
        let mut data = Vec::new();

        'main: loop {
            let tmp = self.identifier(false)?;

            if tmp.is_empty() {
                self.err_op(false, &[">", "<identifier>"])?
            }

            let de = self.expect_char(&[':', '>'])?;
            data.push((tmp, Vec::new()));

            if de == '>' {
                break;
            }

            loop {
                // todo: try to eliminate trt() as it looks redundant and is used only once
                data.last_mut().unwrap().1.push(self.trt()?);

                match self.expect_char(&['+', ',', '>'])? {
                    '+' => {}
                    ',' => break,
                    _ => break 'main,
                }
            }
        }

        Some(data)
    }
}
