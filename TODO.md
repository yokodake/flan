# TODO (code base side)
### `Span`, `Pos`, etc.
Currently everything around lexeme spans and position (inside source_map or source) are a clusterfuck...  
It'd be nice to redesign everything, using types and consistency:  
- [x] ~~decide on `Span::hi` being inclusive or exclusive~~ exclusive range
- [ ] use 3 different newtypes for `Pos`: 
    - [ ] position relative to source_map
    - [ ] position relative to file
    - [x] ~~position in byte or character~~ -> `BytePos`
### `output`
- [ ] refactor `output/mod.rs` into a writer with state.
