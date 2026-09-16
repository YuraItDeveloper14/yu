# Lists

One name keeps one value. But what if there are many: a list of friends, of shopping, of marks?
That is what a list is for.

## A list of values

A list is written in square brackets, with commas between the values. Items are numbered from 1,
so `friends[1]` is the first one.

```yu
friends = ["Olya", "Taras", "Marko"]
say("First:", friends[1])
say("How many:", length(friends))
```

```text
First: Olya
How many: 3
```

`length` says how many items a list holds. For text it says how many letters there are.

## Add to the end

A list can grow while the program runs: `append` puts a value at its end.

```yu
shopping = ["bread"]
append(shopping, "milk")
say(shopping)
```

```text
["bread", "milk"]
```

## Walk the list

`for … in` takes the items one by one and puts each into a name.

```yu
shopping = ["bread", "milk", "apples"]
for item in shopping:
    say("Buy:", item)
```

```text
Buy: bread
Buy: milk
Buy: apples
```

## Count them all

A list and a loop together give you almost any calculation.

```yu
marks = [7, 9, 10, 8]
sum = 0
for mark in marks:
    sum = sum + mark
say("Average:", sum / length(marks))
```

```text
Average: 8.5
```

## Try it yourself

1. Make a list of three favourite dishes and print each one on its own line.

<details>
<summary>Show the answer</summary>

```yu
dishes = ["soup", "pancakes", "dumplings"]
for dish in dishes:
    say(dish)
```

```text
soup
pancakes
dumplings
```

</details>

2. Find the largest number in a list.

<details>
<summary>Show the answer</summary>

```yu
numbers = [3, 17, 8]
largest = numbers[1]
for n in numbers:
    if n > largest:
        largest = n
say("The largest:", largest)
```

```text
The largest: 17
```

</details>
