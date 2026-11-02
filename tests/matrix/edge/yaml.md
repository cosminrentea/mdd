---
type: reference
topic: yaml types
weight: 0.8
count: 3
neg: -7
big: 12345678901234567890
hex: 0x1F
oct: 0o17
flag: true
off: false
yes_str: yes
on_str: on
no_str: no
nothing: ~
empty:
tags: [a, b, "c d"]
nested:
  k: v
  list: [1, 2]
multi: |
  line one
  line two
folded: >
  folded text
  more
date: 2026-11-01
quoted: "x: y"
single: 'it''s'
unicode: "héllo ☃"
anchor: &anc shared
alias: *anc
tagged: !custom value
inf: .inf
nan: .nan
---
# Yaml types

text
