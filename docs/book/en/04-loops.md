# Loops

A computer never gets tired of doing the same thing. A loop is how you say "repeat this" instead
of typing the same lines by hand.

## Repeat

`repeat` runs a block a given number of times.

```yu
stars = ""
repeat 5 times:
    stars = stars + "*"
say(stars)
```

```text
*****
```

## For … from … to

Often the number itself is what you need. `for` counts from the first number to the second, both
ends included, and each time puts the current number into a name.

```yu
for i from 1 to 5:
    say(i, "× 7 =", i * 7)
```

```text
1 × 7 = 7
2 × 7 = 14
3 × 7 = 21
4 × 7 = 28
5 × 7 = 35
```

## Step

When counting by one is not enough, add `step`.

```yu
for i from 0 to 20 step 5:
    say(i)
```

```text
0
5
10
15
20
```

## While

`while` repeats a block as long as its condition is true. Make sure the condition turns false at
some point, or the program never ends. In the Studio such a loop is stopped with the Stop button.

```yu
money = 10
while money >= 3:
    money = money - 3
say("Left", money)
```

```text
Left 1
```

## Break and continue

`break` leaves the loop altogether, and `continue` skips the rest of the block and goes to the
next round.

```yu
for i from 1 to 10:
    if i == 5:
        break
    if i % 2 == 0:
        continue
    say(i)
```

```text
1
3
```

## Try it yourself

1. Print a countdown from 5 to 1.

<details>
<summary>Show the answer</summary>

```yu
i = 5
while i >= 1:
    say(i)
    i = i - 1
```

```text
5
4
3
2
1
```

</details>

2. Add up every number from 1 to 100.

<details>
<summary>Show the answer</summary>

```yu
sum = 0
for i from 1 to 100:
    sum = sum + i
say("Sum:", sum)
```

```text
Sum: 5050
```

</details>
