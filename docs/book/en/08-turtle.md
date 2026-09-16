# The turtle

The shapes of the last chapter appear where you tell them to. There is another way to draw: the
turtle. It crawls over the canvas and leaves a trail behind it.

The turtle starts in the middle of the canvas, looking right, with its pen down.

## Forward and turn

`forward` moves the turtle the way it is looking, and `right` and `left` turn it by a number of
degrees.

```yu
color("blue")
forward(150)
right(90)
forward(150)
```

## A square in a loop

Four identical steps are a job for a loop.

```yu
repeat 4 times:
    forward(120)
    right(90)
```

A full turn is 360 degrees. Divide 360 by the number of corners and you get the turn you need.

## Filling

`begin_fill` tells the turtle to remember its path, and `end_fill` paints the shape it walked
around.

```yu
color("yellow")
begin_fill()
repeat 5 times:
    forward(200)
    right(144)
end_fill()
```

## Pen up

`pen_up` lets the turtle move somewhere else without leaving a trail, and `pen_down` starts
drawing again.

```yu
pen_up()
back(150)
pen_down()
color("green")
repeat 3 times:
    forward(150)
    left(120)
```

## A spiral

Make the step a little longer each time and the line curls up.

```yu
step_length = 5
repeat 40 times:
    forward(step_length)
    right(20)
    step_length = step_length + 3
```

## Try it yourself

1. Draw a triangle with a function that takes the length of a side.

<details>
<summary>Show the answer</summary>

```yu
function triangle(side):
    repeat 3 times:
        forward(side)
        right(120)

color("red")
triangle(150)
```

</details>

2. Draw a star of eight rays coming out of one point.

<details>
<summary>Show the answer</summary>

```yu
color("pink")
repeat 8 times:
    forward(80)
    back(80)
    right(45)
```

</details>
