# StruccDB  

This is (or was) supposed to be a simple document DB kinda like MongoDB but.. well.. struct based.  
Tightly integrated into rust.  

However, as I was finally implementing the "update" portion of CRUD to get a MVP out the door, 
I noticed that updates essentially break the DB. Because of the weird way that (de)serialization into RON 
works, updating the values underlying the serialization causes it to break, because it will reorder fields.  
And apparently, you cannot deserialize a struct if the serialized string's fields are not in the same order 
as the struct.  

As far as I can tell there are two ways to ameliorate this issue:  
1) I use a different (de)serialization strategy. Currently the database server needs to be aware of how structs
are serialized and makes use of arbitrary serialization to inspect and modify documents. This means that
serialization formats like Postcard are out of the picture, but maybe JSON could work?  

2) No updating, only CRD without the U. This could work, but puts the onus on the user to update via delete + create and then use proper key/value combinations in the delete step so as not to accidentally more documents than expected.  

I do not like either of these strategies and don't want to continue with this project. I've tried my best to write readable code, but honestly wtf is this:  

```rust
if let Ok(float) = value.parse::<f64>() {
    return Some(ron::Value::Number(ron::Number::F64(ron::value::F64::new(
        float,
    ))));
}
```

This converts a string into a f64 value type usable by the RON serialization format. But oh my god.. I have no words.  

## Learnings from this project  

So I guess I should write about what this has taught me, but honestly, not much? This is the first time I've used gRPC, so there's that. But most of my work and learnings in this project are RON specific. There isn't even any persistence yet. The "database" is just a in-memory HashMap.  

I think my personal takeaway is that I should use less Rust. I wanted a personal project to have fun and all this project brought me was mysery. 
