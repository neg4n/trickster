<img align="left" width="100" src="./assets/cat.png">

### trickster

user-friendly linux  
memory hacking library  
  
```toml
[dependencies]
trickster = "0.0.6"
```

# Usage and documentation

For example usage of the library, refer to `examples/` directory on this repository.  
[Documentation](https://docs.rs/trickster/) release is both available online on docs.rs and  
offline in `target/doc/` directory after running `cargo doc`.

# Features

This library currently provides ability to:
- Get process id by name.
- Manipulate process memory.
    - Write memory.
    - Read memory.
- Map process memory regions.
    - Find first occurence of memory region with name  
      equal to `x` and optionally permissions equal to `z`.
    
and will provide a lot more in the future.

# Acknowledgements
The cat used in logo comes from [blush.design](https://blush.design/collections/dayflow/stickers-cat/lBGCaheTt)

Thanks to all present and future contributors.  
Library is available under [The MIT License](https://opensource.org/licenses/MIT).

