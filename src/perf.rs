use std::{
    io::{stderr, BufWriter, Write},
    time::{Duration, Instant},
};

use colored::Colorize;

pub struct Perf {
    cycle: usize,
    label: &'static str,
    entries: Vec<(&'static str, [Duration; 3])>,
}

impl Perf {
    pub fn new(label: &'static str, cycle: usize) -> Self {
        Self {
            cycle,
            label,
            entries: Vec::new(),
        }
    }

    pub fn entry<F: FnMut()>(&mut self, id: &'static str, mut fun: F) {
        let mut cycles = Vec::new();

        for _ in 0..self.cycle {
            let time = Instant::now();

            fun();
            cycles.push(time.elapsed());
        }

        cycles.sort_unstable();

        self.entries.push((
            id,
            [
                cycles[0],
                cycles[cycles.len() - 1],
                cycles.iter().sum::<Duration>() / cycles.len() as _,
            ],
        ));
    }

    pub fn finalize(self) {
        let Self {
            label, mut entries, ..
        } = self;
        let mut io = BufWriter::new(stderr().lock());

        io.write(label.underline().to_string().as_bytes()).unwrap();
        io.write(b"\n").unwrap();
        entries.sort_unstable_by_key(|v| v.1[2]);

        for (i, (id, cycles)) in entries.iter().enumerate() {
            io.write(
                format!(
                    "{}. {id} | Best {:.2?} | Worst {:.2?} | Average {:.2?}\n",
                    i + 1,
                    cycles[0],
                    cycles[1],
                    cycles[2]
                )
                .as_bytes(),
            )
            .unwrap();
        }

        let best = entries[0];
        let worst = entries[entries.len() - 1];
        let buf = format!(
            "\n{} {} ({:.2?}) [{:.2}] | {} {} {:.2?}\n\n",
            "Best".underline().bright_green(),
            best.0,
            best.1[2],
            worst.1[2].div_duration_f64(best.1[2]),
            "Worst".underline().bright_red(),
            worst.0,
            worst.1[2]
        );

        io.write(buf.as_bytes()).unwrap();
        io.flush().unwrap()
    }
}
