# Cornetto 🥐

A crate to combine [config](https://crates.io/crates/config) loading and CLI arg parsing with [structops](https://crates.io/crates/structops). Having a more generic solution (like a custom crate providing a handy custom macro) to load parameters (as a Rust struct) with this precedence: CLI args / ENV variables / CONFIG file values. It's also currently not handy to execute Massa binaries from any other directory... as reported in https://gitlab.com/massalabs/massa/-/merge_requests/288


- `#[cornetto(mut, 200)]`: Define a constant variable mutable in test with _reset, with 200 as value
- `#[cornetto(const, 200)]`: Define a simple constant variable, with 200 as value

```rust
use cornetto::Cornetto;

#[allow(dead_code)]
#[derive(Cornetto)]
struct Test {
    #[cornetto(mut, 200)] // mutable on test
    pub price: u64,
    #[cornetto(const, 150)] // always const
    pub const_price: u64,
}

fn main() {
    println!("{}", TEST.price() == 200);
    println!("{}", TEST.const_price() == 200);
}

#[cfg(test)]
mod test {
    use cornetto::Cornetto;
    #[allow(dead_code)]
    #[derive(Cornetto)]
    struct Test {
        #[cornetto(mut, 200)]
        pub price: u64,
        #[cornetto(const, 150)]
        pub const_price: u64,
    }

    #[test]
    fn test_cornetto() {
        assert_eq!(TEST.price(), 200);
        TEST._reset(100); // only accessible from tests
        assert_eq!(TEST.price(), 100);
    }
}
```