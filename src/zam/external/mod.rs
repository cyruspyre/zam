// mod prepare;
// mod r#use;

// use clang::CallingConvention;
// use inkwell::module::Linkage;

// use crate::{function::Function, r#struct::Struct, Parser::Parser};

// #[derive(Debug, Clone)]
// pub struct External {
//     link_type: String,
//     typ: Type
// }

// #[derive(Debug, Clone)]
// pub enum Type {
//     Fn(Function),
//     Struct(Struct),
// }

// impl Parser {
//     pub fn ext(&mut self, ext: &mut Vec<External>) {
//         let lang = self.enclosed('"');
//         let typ = self.until_whitespace();
//         let tmp = lang.as_str();
//         let mut err = || self.err(&format!("cannot use `{typ}` in `{lang}`"));

//         match typ.as_str() {
//             "use" => match tmp {
//                 _ => err(),
//             }
//             "fn" => match tmp {
//                 _ => err(),
//             },
//             _ => self.err("unknown keyword"),
//         };
//     }
// }
