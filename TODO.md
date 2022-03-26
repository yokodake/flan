# TODO (code base side)
### `Span`, `Pos`, etc.
Currently everything around lexeme spans and position (inside source_map or source) are a clusterfuck...  
It'd be nice to redesign everything, using types and consistency:
* decide on `Span::hi` being inclusive or exclusive
* use 3 different newtypes for `Pos`: 
    1. position relative to source_map
    2. position relative to file
    3. position in byte or character
