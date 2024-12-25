use crate::source::Source;

use super::Trait;

pub type Generic = Vec<(String, Vec<Trait>)>;

impl Source {
    pub fn gen(&mut self) -> Generic {
        let mut data = Vec::new();

        'main: loop {
            let tmp = self.word();

            if tmp.is_empty() {
                self.err_op(false, &[">", "<identifier>"])
            }

            let de = self.expect_char(&[':', '>']);
            data.push((tmp, Vec::new()));

            if de == '>' {
                break;
            }

            loop {
                // todo: try to eliminate trt() as it looks redundant and is used only once
                data.last_mut().unwrap().1.push(self.trt());

                match self.expect_char(&['+', ',', '>']) {
                    '+' => {}
                    ',' => break,
                    _ => break 'main,
                }
            }
        }

        data
    }
}
