# beat saber Syntax

## High Level

beat saber is made up of lines. Each line is typically a statement, except in the case of a blank or empty line, in which case it is nothing.

As an example, each of these lines is a statement.

```beatsaber
// puts is not here
// res is "beat saber"
res. // yeet is puts
```

## Statements

Each line (that is not a comment) is perceived as a statement.

Statements are made up of a behaviour (represented after a `//`) and an optional series of expressions.

All statements in beat saber can be likened to bindings or assignment.

From the hello world example:

```beatsaber
// puts is not here
// res is "beat saber"
res. // yeet is puts
```

Note that the first statement is an assignment statement to the identifier: `puts`, describing it as an external function that is `not here`.

The second statement is an assignment statement that assigns `res` the value of the string literal described by `"beat saber"`. More specifically, `res` is a pointer to the front of the array of characters that represent the string literal.

The third statement is an assignment statement that discards the value that would otherwise be assigned (`yeet`) with the result being a call expression that calls `puts` with a single parameter being `res`.

For clarity, the above would essentially boil down to the following C:

```c
char* res = "beat saber";
puts(res);
```

## Expressions


## Behaviours


