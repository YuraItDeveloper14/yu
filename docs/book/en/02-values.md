# Numbers and text

In the first chapter Yu printed what we wrote in quotes. Now we will keep values, count with them
and ask the person a question.

## Values have names

A name remembers a value. The `=` sign puts the value into the name:

```yu
age = 15
say("I am", age, "years old")
```

```text
I am 15 years old
```

The name is always on the left of `=` and the value on the right. Names may be written in
Ukrainian too, and they may hold an apostrophe: `ім'я`.

## Counting

Yu counts with `+`, `-`, `*` (times), `/` (divide) and `%`, the remainder after dividing.

```yu
sum = 7 + 8
say("Sum:", sum)
say("Half:", sum / 2)
say("Remainder after dividing by 4:", sum % 4)
```

```text
Sum: 15
Half: 7.5
Remainder after dividing by 4: 3
```

## Text joins up

The same `+` means "join" for text. A number can be joined to text as well.

```yu
name = "Olya"
say("Hi, " + name + "!")
```

```text
Hi, Olya!
```

Look at the space inside `"Hi, "`: without it the words would stick together.

## Ask the person

`ask` shows a question and waits for an answer. In the Studio the question appears in the
terminal, and that is where the answer is typed.

```input
Olya
```

```yu
name = ask("What is your name?")
say("Hi, " + name + "!")
```

```text
Hi, Olya!
```

## Text and numbers are different

`ask` always gives back text, even when the person typed digits. To count with it, turn it into a
number with `number`. The command `text` goes the other way.

```input
12
```

```yu
years = number(ask("How old are you?"))
say("Next year you turn", years + 1)
```

```text
Next year you turn 13
```

## Round and random

`round` gives the nearest whole number.

```yu
say("2.5 rounds to", round(2.5))
```

```text
2.5 rounds to 3
```

`random` gives a different number every time, so there is no output here: run it and see yours.

```yu
say("Dice:", random(1, 6))
```

## Try it yourself

1. Ask for two numbers and print their sum.

<details>
<summary>Show the answer</summary>

```input
4
5
```

```yu
first = number(ask("First number:"))
second = number(ask("Second number:"))
say("Sum:", first + second)
```

```text
Sum: 9
```

</details>

2. Put the name of your city into a name and greet it.

<details>
<summary>Show the answer</summary>

```yu
city = "Kyiv"
say("Hi, " + city + "!")
```

```text
Hi, Kyiv!
```

</details>
