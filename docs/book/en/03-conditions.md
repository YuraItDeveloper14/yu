# Conditions

So far the program ran every line in turn. A condition lets it choose: do something, or skip it.

## If

`if` checks a condition. The lines below it, indented by four spaces, run only when the condition
is true.

```yu
temperature = 25
if temperature > 20:
    say("It is warm today")
```

```text
It is warm today
```

The colon at the end of the line opens a block, and the indent shows what belongs to it. You can
compare with `>` greater, `<` less, `>=` greater or equal, `<=` less or equal, `==` equal and
`!=` not equal.

One `=` puts a value into a name; two `==` compare. They are different things.

## Else, else if

When the condition is false, the block under `else` runs. Between them you may put as many
`else if` blocks as you like: Yu takes the first one whose condition is true.

```yu
score = 7
if score >= 10:
    say("Excellent")
else if score >= 6:
    say("Good")
else:
    say("Almost there")
```

```text
Good
```

## And, or, not

Conditions can be combined. `and` is true when both parts are true, `or` when at least one is,
and `not` turns the answer around.

```yu
age = 12
if age > 6 and age < 18:
    say("Goes to school")
say("Not grown up yet:", not (age >= 18))
```

```text
Goes to school
Not grown up yet: true
```

## True, false, nothing

A comparison gives one of two values: `true` or `false`. There is also `nothing`, which is how Yu
says that there is no value.

```yu
rain = false
say("Rain:", rain)
found = nothing
say("Found:", found)
```

```text
Rain: false
Found: nothing
```

## Try it yourself

1. Check whether a number is even. A hint: an even number divides by 2 with no remainder.

<details>
<summary>Show the answer</summary>

```yu
number_to_check = 10
if number_to_check % 2 == 0:
    say("Even")
else:
    say("Odd")
```

```text
Even
```

</details>

2. A ticket costs 50, and children under 12 go free. Print the price for the age 10.

<details>
<summary>Show the answer</summary>

```yu
age = 10
if age < 12:
    say("Price: 0")
else:
    say("Price: 50")
```

```text
Price: 0
```

</details>
