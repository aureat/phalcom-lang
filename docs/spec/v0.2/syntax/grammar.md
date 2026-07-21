# Grammar (Consolidated)

Part of the [Phalcom Language Specification](../README.md). Status: Draft 0.1.

This appendix collects the full lexical and syntactic grammar of the v0.2 target
in one place. The sibling files (`lexical.md`, `expressions.md`,
`statements-and-declarations.md`) are the annotated walkthroughs of the pieces
assembled here; `../implementation-status.md` tracks where the built compiler
currently diverges from this target grammar.

Notation key: `:=` define · `|` alternative · `[ ]` optional · `{ }` zero-or-more
· `( )` grouping · `"lit"` literal terminal · UPPERCASE identifier is a lexical
token · `(* *)` comment.

```ebnf
(* ================================================================ *)
(* Program                                                           *)
(* ================================================================ *)

program        := { top_item } EOF
top_item       := class_decl | import_decl | statement

(* ================================================================ *)
(* Declarations                                                      *)
(* ================================================================ *)

import_decl    := "import" IDENT
                 | "import" IDENT { "," IDENT } "from" IDENT
                 | "import" IDENT "as" IDENT

class_decl     := "class" IDENT [ "extends" IDENT ] "{" { member } "}"
member         := { attribute } member_body
attribute      := "@" IDENT [ "(" [ arg { "," arg } ] ")" ]
member_body    := setter_def | method_def | getter_def | field_init

setter_def     := IDENT "=" param_list method_body
method_def     := method_name param_list method_body
getter_def     := IDENT method_body
field_init     := FIELD "=" expr

method_name    := IDENT | operator
operator       := "+" | "-" | "*" | "/" | "%"
                 | "&" | "|" | "^" | "~" | "<<" | ">>"
                 | "==" | "!=" | "<" | "<=" | ">" | ">="
                 | "and" | "or" | "is"
method_body    := "=>" expr | block

param_list     := "(" [ param { "," param } ] ")"
param          := "*" IDENT              (* variadic *)
                 | IDENT IDENT ":"        (* external label + internal name + type *)
                 | IDENT ":"              (* name + type *)
                 | IDENT                  (* bare name *)

(* ================================================================ *)
(* Statements                                                        *)
(* ================================================================ *)

statement      := binding | return_stmt | if_stmt | while_stmt | for_stmt
                 | break_stmt | continue_stmt | throw_stmt | try_stmt | expr_stmt

binding        := ( "let" | "const" ) IDENT [ "=" expr ]
return_stmt    := "return" [ expr ]
break_stmt     := "break"
continue_stmt  := "continue"
expr_stmt      := expr

if_stmt        := "if" "(" expr ")" block [ "else" ( block | if_stmt ) ]
while_stmt     := "while" "(" expr ")" block
for_stmt       := "for" "(" IDENT "in" expr ")" block

throw_stmt     := "throw" expr
try_stmt       := "try" block { on_clause } [ catch_clause ] [ ensure_clause ]
on_clause      := "on" IDENT IDENT block
catch_clause   := "catch" IDENT block
ensure_clause  := "ensure" block

(* ================================================================ *)
(* Expressions (precedence low -> high; binary ops are left-assoc    *)
(* except assignment and "??", which are right-assoc)                *)
(* ================================================================ *)

expr           := assignment
assignment     := postfix ( "=" | "+=" | "-=" | "*=" | "/=" | "%=" ) assignment
                 | coalesce
coalesce       := or_expr [ "??" coalesce ]
or_expr        := and_expr { "or" and_expr }
and_expr       := equality { "and" equality }
equality       := comparison { ( "==" | "!=" ) comparison }
comparison     := bit_or { ( "<" | "<=" | ">" | ">=" | "is" ) bit_or }
bit_or         := bit_xor { "|" bit_xor }
bit_xor        := bit_and { "^" bit_and }
bit_and        := shift { "&" shift }
shift          := additive { ( "<<" | ">>" ) additive }
additive       := multiplicative { ( "+" | "-" ) multiplicative }
multiplicative := unary { ( "*" | "/" | "%" | "~/" ) unary }
unary          := ( "-" | "~" | "!" | "not" ) unary | postfix

postfix        := primary { "." ( IDENT | keyword ) [ arg_list ]  (* send / property *)
                           | "?." IDENT [ arg_list ]              (* optional send *)
                           | arg_list                             (* call -> .call *)
                           | block                                (* trailing block *)
                           | "::" ( "#" symbol_sel | IDENT ) }    (* method reference *)

arg_list       := "(" [ arg { "," arg } [ "," ] ] ")"
arg            := [ IDENT ":" ] [ "*" ] expr

(* ================================================================ *)
(* Primaries                                                         *)
(* ================================================================ *)

primary        := literal | grouping | tuple | list | map | block
                 | symbol | "self" | "super" | IDENT | FIELD

grouping       := "(" expr ")"
tuple          := "(" ")"
                 | "(" expr "," ")"
                 | "(" expr "," expr { "," expr } [ "," ] ")"
list           := "[" [ expr { "," expr } [ "," ] ] "]"
map            := "{" map_entry { "," map_entry } [ "," ] "}"
map_entry      := ( IDENT | expr ) ":" expr

block          := "{" [ block_params "=>" ] { statement } "}"
                 | IDENT "=>" expr
block_params   := IDENT { "," IDENT }

literal        := INT | FLOAT | STRING | "true" | "false"

symbol         := "#" IDENT | "#" symbol_sel
symbol_sel     := IDENT "(" [ slot { "," slot } ] ")" | operator
slot           := "_" | IDENT

(* Operator selector symbols are bare: `#+`, `#&`, `#~`. `#~` denotes the
   nullary `~()` selector; every other operator denotes its canonical arity. *)

(* --- Lexical --- *)

(* identifiers, field slots, numbers *)
IDENT          := LETTER { LETTER | DIGIT }
FIELD          := "_" { LETTER | DIGIT }
INT            := DEC-INT | BIN-INT | OCT-INT | HEX-INT
FLOAT          := DEC-DIGITS "." DEC-DIGITS [ EXPONENT ]
                | "." DEC-DIGITS [ EXPONENT ]
                | DEC-DIGITS EXPONENT
DEC-INT        := ZERO-INT | NZ-DIGIT { DEC-GROUP }
ZERO-INT       := "0" { "0" | "_" "0" }
BIN-INT        := "0" ( "b" | "B" ) [ "_" ] BIN-DIGIT { BIN-GROUP }
OCT-INT        := "0" ( "o" | "O" ) [ "_" ] OCT-DIGIT { OCT-GROUP }
HEX-INT        := "0" ( "x" | "X" ) [ "_" ] HEX-DIGIT { HEX-GROUP }
EXPONENT       := ( "e" | "E" ) [ "+" | "-" ] DEC-DIGITS
DEC-DIGITS     := DIGIT { DEC-GROUP }
DEC-GROUP      := DIGIT | "_" DIGIT
BIN-GROUP      := BIN-DIGIT | "_" BIN-DIGIT
OCT-GROUP      := OCT-DIGIT | "_" OCT-DIGIT
HEX-GROUP      := HEX-DIGIT | "_" HEX-DIGIT
NZ-DIGIT       := "1".."9"
BIN-DIGIT      := "0" | "1"
OCT-DIGIT      := "0".."7"
HEX-DIGIT      := DIGIT | "a".."f" | "A".."F"

(* strings and interpolation *)
STRING         := '"' { string_char | interpolation } '"'
string_char    := escape | ANY_CHAR   (* ANY_CHAR: any source character other
                                          than '"', "\", or a raw NEWLINE *)
escape         := "\\" ( '"' | "\\" | "n" | "t" | "r" )
                 (* partial set — full escape repertoire unresolved, see Notes *)
interpolation  := "\(" expr ")"

(* character classes *)
LETTER         := "A".."Z" | "a".."z"
DIGIT          := "0".."9"

(* trivia: insignificant, may appear between any two tokens; not part of the
   syntactic derivation from `program` *)
NEWLINE        := "\n" | "\r\n"
line_comment   := "//" { any_char_but_newline }
block_comment  := "/*" { any_char } "*/"

(* end of input, emitted by the lexer as a sentinel; atomic *)
(* EOF is a terminal with no further decomposition *)

(* reserved words recognized in the position postfix expects a message/
   property name (e.g. `.self`, `.and`); "extends", "try", "catch", "on",
   "ensure" are contextual keywords, only reserved in class/try position;
   "fn" is reserved-inactive, not currently a keyword *)
keyword        := "let" | "const" | "class"
                 | "self" | "super" | "if" | "else" | "while" | "for" | "in"
                 | "break" | "continue" | "return" | "and" | "or" | "not" | "is"
                 | "true" | "false" | "import" | "as" | "throw"
                 | "extends" | "try" | "catch" | "on" | "ensure"

(* punctuation / operators, used directly as literals above; listed here for
   reference only — not a nonterminal:
   +  -  *  /  %  ~/  &  |  ^  ~  <<  >>  ==  !=  <  <=  >  >=
   =  +=  -=  *=  /=  %=  ??  ?.  .  ::  :  =>  ( )  { }  [ ]  ,  ;  #  @  !
   reserved-inactive: ..  ...  ->
   note: a lone "?" (without "." or "?") is not a token *)
```

## Notes

- **`.. ` / `...` range operators** — reserved-inactive; no range literal syntax
  is wired into this grammar yet. See `../core/tuple-and-range.md`.
- **`#{...}` set literal** — reserved-inactive; sets are not yet a distinct
  literal surface. See `../core/map-and-set.md`.
- **`->`** — reserved-inactive; not used for blocks, types, or returns in this
  grammar. See `../selectors.md#7`.
- **`fn`** — reserved-inactive; no alternate function-declaration keyword is
  active alongside `method_def`/`construct_def`. See `../lexical-structure.md`.
- **Full string-escape set** — the escape production above is illustrative; the
  only escapes the spec fixes today are `\\` and `\(` (interpolation), and the
  complete repertoire (unicode escapes, etc.) is unresolved. See
  `../lexical-structure.md#5` and
  [ADR-0022](../../../adr/0022-string-interpolation-backslash-paren-sigil.md).
- **Default arguments** — `param` has no `= expr` default-value form yet; how
  defaults interact with selector arity encoding is unresolved. See
  `../selectors.md#7`.
- **`is` / `as` full semantics** — `is` appears as a comparison/method-overload
  operator and `as` as an import-aliasing keyword here, but their full
  type-test/conversion semantics are unresolved. See `../selectors.md#7`.
- **Range operator precedence** — no range operator is active in `additive`/
  `multiplicative` yet, so its eventual precedence slot is unresolved. See
  `../core/tuple-and-range.md`.
