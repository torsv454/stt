# stt
STT us a simple (really, super simple) text template engine in Rust. I've written it as part of my journey into learning Rust. 

Sample usage: 
```rust
let template = stt::Template::new("Hello $who$!");
let lookup = |k: &str| if k == "who" {
    Some(String::from("world"))
} else {
    None
};
assert_eq!(template.render(&lookup),"Hello world!");
```

