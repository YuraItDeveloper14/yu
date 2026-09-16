# When something goes wrong

An error is not a disaster. It is Yu telling you where it stopped and what it did not understand.
In the Studio the message appears in the terminal, and the place itself is underlined in the code.

Every error has three parts: the line where it happened; that line with arrows under the problem;
and sometimes a hint about what was probably meant.

## Unknown name

The most common mistake is a typo in a command's name.

```yu-error
sa("Hello")
```

```text
Error on line 1: unknown name 'sa'
  1 | sa("Hello")
    | ^^
  Did you mean 'say'?
```

The last line is the hint. Yu compared the name with the ones it knows and found the closest.

## An unclosed quote

Text starts with a quote and ends with one. When the second is missing, Yu reads to the end of
the line and finds no pair.

```yu-error
say("Hello)
```

```text
Error on line 1: text is missing its closing quote "
  1 | say("Hello)
    |     ^^^^^^^
```

## A forgotten indent

After a colon Yu waits for an indented block. Without the indent it cannot tell what belongs to
the condition.

```yu-error
if 5 > 3:
say("Yes")
```

```text
Error on line 2: expected an indented block on the next line
  2 | say("Yes")
    | ^^^
```

An indent is four spaces. The Studio types them for you when you press Enter after a colon.

## Division by zero

Some mistakes show up only while the program runs: the lines are right, but the action is
impossible.

```yu-error
say(10 / 0)
```

```text
Error on line 1: division by zero
  1 | say(10 / 0)
    |     ^^^^^^
```

## How to read an error

Look at the line number and the arrows first: they show the exact place. Then read the message
itself. When there is a hint, it is usually the answer. And remember that an error stops the
program where it was noticed, while the cause is sometimes a line above.

## Try it yourself

1. Find the mistake and fix the program `say("Hello"`.

<details>
<summary>Show the answer</summary>

The closing bracket is missing.

```yu
say("Hello")
```

```text
Hello
```

</details>

2. This program was meant to print 1, 2, 3, but Yu shows an error. Fix it.

```yu-error
for i from 1 to 3:
say(i)
```

```text
Error on line 2: expected an indented block on the next line
  2 | say(i)
    | ^^^
```

<details>
<summary>Show the answer</summary>

The line inside the loop needs its indent.

```yu
for i from 1 to 3:
    say(i)
```

```text
1
2
3
```

</details>
