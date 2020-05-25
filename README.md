# trickster
  
<sub> user-friendly linux memory hacking library written in Rust. </sub>
  
This project is continuation of *(from now)* abandoned [tr](https://github.com/neg4n/tr)  
library with the same purpose but written in C++17.
  
```toml
[dependencies]
trickster = "0.0.4"
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
Thanks to all present and future contributors.  
Library is available under [The MIT License](https://opensource.org/licenses/MIT).
