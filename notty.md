# Notty Escape Codes

Notty escape codes are a set of escape-initiated command sequences, intended
to be transmitted inline with UTF8 encoded textual data representing the input
from and output to a virtual terminal program. They are analogous to so-called
ANSI escape codes, defined in ECMA-48, and implement much of the same
functionality.

## Structure of Notty Escape Codes

All Notty escape codes have the same structure. Written in psuedo-BNF:

```
<CODE>          ::= "\x1b{" <OPCODE> (";" <ARGUMENT>)* ("{" <ATTACHMENT>)* "}"
<OPCODE>        ::= <INTEGER>
<ARGUMENT>      ::= <INTEGER> ("." <INTEGER>)*
<ATTACHMENT>    ::= n @ <INTEGER> ";" <.>{n}
<INTEGER>       ::= [0-9A-Fa-f]+
```

The sequence initializer is the string U+1b,U+7b (the escape character and the
left curly brace). This sequence is fully disambuguates notty escape codes from
any standardized ANSI escape codes, which do not ever begin with these
charachters.

The next component of the code is a hexademically encoded opcode. Every valid
command is assigned an opcode, which defines what subsequent components are
a validly structured commands. Escape codes beginning with unknown op codes
should be entirely ignored.

This is followed by two sections: the "arguments" section, which is a sequence
of arguments to the op code which are encoded in a simple, consistent format,
and the "attachments" section, which are arbitrary sequences of bytes. The
command is terminated when the "}" character appears after the end of an
attachment.

### Arguments

Each opcode has a sequence of arguments which is expected to be transmitted
with it. These arguments are well-typed values, and each opcode has only
one valid arity of arguments it can receive (modulated by the note below about
default values).

An argument is encoded as a sequence of period separated hexadecimal integers;
the arguments are separated from one another and from the op code by the
semicolon character.

Opcodes may have default arguments. If all remaining arguments for this
command are the default values, these arguments can be omitted entirely from
the code. If one argument is default, but subsequent arguments are not, and
that argument is not a boolean value, it can be encoded as the '0' character,
which is reserved for default arguments for all types except booleans.

The argument types are:

__TODO__

### Attachments

Some opcodes take attachments as well as or instead of arguments. Attachments
have no general type, but each command expects attachments to be encoded in
a format specified by the description of that command.

Attachments come subsequent to any arguments, and each attachment is initiated
by the '{' character. The first component of an attachment is a hex integer
which is terminated by the ';' character. This integer is interpreted as a
length, and the number of bytes equivalent to its length read after the ';'
character are the data associated with that attachment.

For example, the first attachment to the "PUT IMAGE AT" opcode is a MIME type,
which determines how the data in the second attachment will be interpreted.
The first attachment for a png would therefore be encoded as "{9;image/png",
because "image/png" is 9 bytes long.

## Op Codes

__TODO__
