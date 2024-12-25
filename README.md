A general purpose programming language

```rs
extern "C" use {
    "stdio" { printf }
    "stdlib" { malloc, free }
}

fn main() {
    let mut vec: *u10 = malloc(sizeof(u10) * 10);

    for n in 0..10 {
        let one = vec[n];

        if one? {
            *one = 10
        }

        vec[n + 1]
    }

    free(vec)
}
```