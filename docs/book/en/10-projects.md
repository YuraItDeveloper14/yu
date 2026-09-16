# Three projects

Here are three finished programs. Each is built from what the book already covered: conditions,
loops, lists, functions and drawing. Run one, then change it for yourself — that is how it sticks.

## Guess the number

The program has a number in mind and you guess it. `while` repeats the question until the answer
matches.

```input
5
7
```

```yu
secret = 7
guess = number(ask("Guess a number from 1 to 10:"))
while guess != secret:
    say("Not that one. Try again")
    guess = number(ask("Once more:"))
say("You got it!")
```

```text
Not that one. Try again
You got it!
```

Replace `7` with `random(1, 10)` and the program picks a new number every run.

## A house

The walls are rectangles, and the roof is drawn by the turtle with a fill. First it walks quietly
to the top left corner of the wall, and only then goes around the triangle.

```yu
background("lightblue")
color("green")
rect(0, 300, 600, 100)
color("white")
rect(220, 180, 160, 120)
color("red")
pen_up()
back(80)
left(90)
forward(20)
right(90)
pen_down()
begin_fill()
repeat 3 times:
    forward(160)
    left(120)
end_fill()
```

Add a sun with a circle, a door with a rectangle, a label with the street name — the canvas is
yours.

## A quiz

Every right answer adds a point. Text is compared the same way as numbers: with `==`.

```input
Kyiv
4
```

```yu
score = 0
if ask("The capital of Ukraine?") == "Kyiv":
    score = score + 1
if number(ask("2 + 2 = ?")) == 4:
    score = score + 1
say("Your score:", score, "out of 2")
```

```text
Your score: 2 out of 2
```

## Try it yourself

1. Add a third question to the quiz.

<details>
<summary>Show the answer</summary>

```input
Kyiv
4
Yu
```

```yu
score = 0
if ask("The capital of Ukraine?") == "Kyiv":
    score = score + 1
if number(ask("2 + 2 = ?")) == 4:
    score = score + 1
if ask("What is this language called?") == "Yu":
    score = score + 1
say("Your score:", score, "out of 3")
```

```text
Your score: 3 out of 3
```

</details>

2. Count how many guesses it took.

<details>
<summary>Show the answer</summary>

```input
5
7
```

```yu
secret = 7
guesses = 1
guess = number(ask("Guess a number from 1 to 10:"))
while guess != secret:
    guesses = guesses + 1
    guess = number(ask("Once more:"))
say("You got it in", guesses, "guesses")
```

```text
You got it in 2 guesses
```

</details>
