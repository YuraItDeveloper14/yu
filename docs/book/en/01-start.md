# Your first program

Yu runs in your browser: open [yu-lang.vercel.app](https://yu-lang.vercel.app) and start writing.
There is nothing to install.

The editor is on the left of the Studio; that is where the program goes. On the right there are
two tabs: Picture shows what the program drew, Terminal shows what it said in words. The Run
button at the top right starts the program, and so does Ctrl+Enter.

## Say something

The shortest program in Yu is one word, `say`, and some text in brackets:

```yu
say("Hello, world!")
```

```text
Hello, world!
```

Part by part. `say` is a command: write this in the terminal. The brackets hold what to say. The
quotes mean that what is inside is text, not the name of something. Press Run and the Studio
opens Terminal with your line.

## More than one line

A program runs from top to bottom, line by line. A line that starts with `#` is not run: it is a
comment, a note for a person.

```yu
# Yu does not run this line
say("My name is Yurii")
say("I am 15 years old")
```

```text
My name is Yurii
I am 15 years old
```

## Yu can count

The brackets can hold arithmetic as well as text. Separate several values with commas: Yu prints
them on one line with a space between them.

```yu
say("2 + 2 =", 2 + 2)
```

```text
2 + 2 = 4
```

Text in quotes is printed as it is, and arithmetic without quotes is worked out. That is why
`"2 + 2 ="` stayed a label and `2 + 2` became `4`.

## Try it yourself

1. Write a program that prints two lines about you: your name and where you live.

<details>
<summary>Show the answer</summary>

```yu
say("My name is Olya")
say("I live in Kyiv")
```

</details>

2. Let Yu work out 7 × 8 and print the result with a label.

<details>
<summary>Show the answer</summary>

```yu
say("7 × 8 =", 7 * 8)
```

```text
7 × 8 = 56
```

</details>
