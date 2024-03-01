# hyper

## test
``` bash
cargo test -- --nocapture
```

## Object File format
as defined by Linkers and Loaders book

```
LINK
nsegs nsyms nrels
-- segments --
-- symbols --
-- rels --
-- data --
```

### Segments
Where `-- segments --` is `nsegs` lines in format:

```
<name> <starting address> <length in bytes> <type (R,W,P)>
```
e.g.:

```
.text 1000 2500 RP
.data 4000 c00 RWP
.bss  5000 1900 RW
```

### Symbols
Where `-- symbols --` is `nsyms` lines in format:

```
<name> <value> <seg num> <type (D/U)>
```

### Relocations
Where `-- rels --` is `nrels` lines in format:

```
<loc> <seg> <ref> <type>
```
`loc` to be relocated
`seg` in which the `loc` is found
`ref` is the segment or symbol number to be relocated there
`type` can be e.g. 4A for 4-byte absolute address of R4 for 4-byte relative (arch dependent)

### Data
single hexstring

