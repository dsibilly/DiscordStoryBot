supported [features](https://github.com/inkle/ink/blob/master/Documentation/WritingWithInk.md):

Differences from the official implementation:

* Fallbacks are always sticky, to prevent runtime errors.
* Only the checked-off items below are supported.

## v0.1

- [x] content
- [x] comments
- [x] tags
  - [x] multiple same-line tags
  - [x] "Tags for a line can be written above it, or on the end of the line"
  - [x] tags above the first line of a knot are also tags on that knot
  - [x] global tags (at the very top of the main ink file)
- [x] choices (+)
- [x] sticky choices as special (+)
- [x] choices (*)
- [x] knots
- [x] knot titles with trailing ='s
- [x] diverts
- [x] choices, content, and diverts on the same line
- [x] stitches
- [x] local diverts
- [x] fallback choice (choice without choice text)
- [x] choices where you don't show the text

## v0.2
- [x] `[?]`
  - `both [only pre-choice] only post-choice` 
- [x] conditional choices `{}`
  - [x] "test knot_name is true if any stitch inside that knot has been seen." (whenever you go to a stitch inside a knot, when you were not already in that knot)
- [ ] global constants `CONST`
  - notes: DialogLine needs to always have `\n`s , so we can interleave other things, yeah?
- [ ] global variables `VAR`
- [ ] logical operators AND `&&`, OR `||`, and NOT `not`
- [ ] not as exclamation point: `!`
- [ ] integer comparison checks `{seen_clue > 3}`
- [ ] variable text `{one|two|three}`
- [ ] conditional text `{variable: text if true|text if false}`
- [ ] numerical maths and logic `~ x = (x*x) - (y*y)`

## v0.3
- [ ] same-line diverts mean no newline after "Diverts are invisible"
- [ ] glue (though maybe this is more part of the story runner?)
- [ ] includes
- [ ] alternatives: sequences `|`
- [ ] alternatives: cycles `&`
- [ ] alternatives: once-only `!`
- [ ] alternatives: shuffles `~`
- [ ] alternatives: blank elements
- [ ] alternatives: nested
- [ ] alternatives: divert statements
- [ ] alternatives: inside choice text
- [ ] alternatives: escaping `{` with backslash
- [ ] CHOICE_COUNT()
- [ ] TURNS()
- [ ] TURNS_SINCE()
- [ ] SEED_RANDOM()
- [ ] storing diverts as variables
- [ ] printing variables
- [ ] evaluating strings
- [ ] RANDOM()
- [ ] INT() FLOOR() FLOAT()
- [ ] string comparison `==`, `!=`, `?`
- [ ] conditional blocks `if`, `else`
- [ ] switch blocks
- [ ] temporary variables
- [ ] knot and stitch parameters

## v0.4

- [ ] functions
- [ ] tunnels
- [ ] threads
- [ ] lists (TODO: split this into sub-sections)
- [ ] Weave: gathers
- [ ] Weave: nested flow
- [ ] Weave: nested gather points
- [ ] Weave: labelled gather points and options `- (label)`
- [ ] Weave: scope
]