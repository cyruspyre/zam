mod prepare;
mod r#use;

use crate::{function::Function, source::Source};

#[derive(Debug)]
pub enum External {
    Fn(Function),
}

impl Source {
    pub fn ext(&mut self, ext: &mut Vec<External>) {
        let lang = self.enclosed('"');

        if !matches!(lang.as_str(), "C") {
            self.err("only valid external type is `C`")
        }

        match self.expect(&["fn", "use", "struct"]).as_str() {
            // seperate use to seperarate mod
            "use" => self.ext_use(ext),
            _ => todo!(),
        }

        println!("{:?}", ext)
    }
}
