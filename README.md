Experimental general purpose programming language

```rs
extern "C" use {
    "stdio" { printf }
    "stdlib" { malloc, free }
}

fn main() {
    let arr = malloc<*u20>(sizeof(u20) * 10);

    for n in 0..10 {
        arr[n] = 10 + 10 * n
    }

    printf("%s\n", Vec::from_raw_ptr(arr, 10).to_string());
    free(arr);
}
```