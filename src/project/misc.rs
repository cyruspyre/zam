use crate::{misc::Either, parser::Parser, zam::Zam};

impl Zam {
    /// Get formatted location of the current file with the given range/span
    /// e.g `src/main.z:16:25`
    pub fn location(&self, rng: [usize; 2]) -> String {
        let Parser { path, line, .. } = &self.parser;
        let idx = line.binary_search(&rng[0]).either();

        format!(
            "{}:{}:{}",
            path.display(),
            idx + 1,
            rng[1] - line.get(idx.wrapping_sub(1)).unwrap_or(&0)
        )
    }
}
