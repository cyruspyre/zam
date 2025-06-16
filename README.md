# Zam
An experimental general-purpose programming language inspired by Rust, but more relaxed. Featuring static typing, generics, traits, and more.
## Planned Features
- C interop with minimal pain as much as possible
- Interpreter and JIT mode for fast development cycle
- Compile Time Evaluation for niche cases
## Status
Currently, Zam is far from it's usable state. Type checking is really really barebone. Only syntax level things are done. Not to mention, I still have to deal with LLVM.
## Building/Running
As with most Rust projects, you just do `cargo run` or `cargo build`. However, you might come accross LLVM related issues. For that, I suggest referring to the Inkwell docs.
## Why?
First and foremost, I'm a hardcore Rust glazer. As expected, I had this extreme delulu of writing my own cross platform code editor so I went ahead and failed miserably. This language is also a pure delulu. Wanting something like Rust but with extra niche things that can also tend to my future plans, which to my knowledge doesn't exist as a language. Rust elites might shred me but I'm not a big fan of how the lifetimes are implemented in it. Imagine having to specify lifetime specifiers all over just because you had to store a reference in a struct. Maybe I'm not seeing the bigger picture, but for now, I'm curious as to where Zam goes.
## License
This project is licensed under MIT licence. See the [LICENSE](LICENSE) file for details.
## Contributing
While Zam is in early development, discussions and feedback are welcome! Feel free to:
- Open issues for bugs or feature suggestions
- Join discussions about language design
- Watch the repository for updates