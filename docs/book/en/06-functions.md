# Functions

When the same piece of a program is needed in several places, you give it a name. A named piece
like that is called a function.

## Give a block a name

`function` starts the description: a name, and in the brackets the parameters, the values the
function will be given when it is called. The indented lines are its body. On its own the body
does not run: it waits to be called.

```yu
function greet(name):
    say("Hi, " + name + "!")

greet("Olya")
greet("Taras")
```

```text
Hi, Olya!
Hi, Taras!
```

## Return an answer

A function can do more than print: it can hand a value back. That is what `return` does.

```yu
function area(width, height):
    return width * height

say("The room's area:", area(4, 3))
```

```text
The room's area: 12
```

After `return` the function ends at once, even when more lines follow.

## A function can call itself

A function may call itself inside its own body. What matters is that there is a case where it
stops calling and simply returns an answer.

```yu
function factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

say("5! =", factorial(5))
```

```text
5! = 120
```

## Try it yourself

1. Write a function that returns the smaller of two numbers.

<details>
<summary>Show the answer</summary>

```yu
function smaller(a, b):
    if a < b:
        return a
    return b

say(smaller(9, 4))
```

```text
4
```

</details>

2. Write a function that prints a word a given number of times.

<details>
<summary>Show the answer</summary>

```yu
function repeat_word(word, times_to_say):
    repeat times_to_say times:
        say(word)

repeat_word("Yu", 3)
```

```text
Yu
Yu
Yu
```

</details>
