# Subdividing the screen in notty

notty supports subdividing a screen into multiple sections, each of which can
contain independent grids. Each of these grids can retain off-screen state,
scroll independently, be resized independently, and so on. There are several
commands which manipulate this feature, but what's most important is
understanding the underlying layout model that notty uses. Here are the
rules that notty's model uses to make screen subdivision easier:

1. The screen is subdivided into nested, rectangular sections.
2. Each section of the screen contains a stack of panels.
3. Each panel is either a single character grid, or is split into two smaller
   sections, each of which contain a stack of panels.

# For example

Consider this 6x6 screen:

```
  0 1 2 3 4 5
 +-----+-----+
0|     |     |
1|     |     |
2|     |     |
3|     |     |
 +-----+-----+
4|           |
5|           |
 +-----+-----+
```

Though this screen contains only 3 grids, it actually contains 5 sections:

1. The base section, which contains the entire 6x6 grid. The panel in this
   section is split horizontally between rows 3 and 4.
2. The top portion of the base section, filling the area from 0,0 to 5,3
   (inclusive). The panel in this section is split vertically between columns 2
   and 3.
3. The left portion of the previous section, from 0,0 to 2,3 (inclusive). The
   panel here contains a character grid.
4. The right portion of that section, from 3,0 to 5,3 (inclusive). The panel
   here contains a character grid as well.
5. The lower portion of the base section, from 0,4 to 5,5 (inclusive). The
   panel here also contains a character grid.

# Commands that can be applied to any section

In the implementation, each of these sections of the screen has an identifying
tag, so that different commands can be applied to each section. These actions
can all be performed on any section of the screen, whether it contains a grid
or a split:

## Pushing a Panel

Sections don't just contain one Panel, they contain a stack of Panels. You can
push a new Panel, which contains an empty grid, over any section on the screen. 

This includes sections which contain split panels, and also includes the 'base
section' of the entire screen. This means, for example, a new grid could be
pushed over the entire screen in the above example, or over the top half of
the screen that contains the vertical split.

## Popping a Panel

The top panel of a section can be popped off, deleting whatever it contained.
This is only true if a section contains more than one panel - a section with a
stack with only one member will not change when the pop command is applied.

The tag of each section is assigned when that section is created, but the tag
of the base section is always 0. If two sections are given the same tag, the
behavior of notty commands applied to that tag is implementation defined, and
you should not create sections with the same tag.

## Rotating the Panel Stack

The stack in each section can also be rotated up and down, switching which
panel is on top (and therefore visible) without deleting any panels. The stack
cannot be arbitrarily reordered, it can only be rotated in either direction.

## Splitting a Section

Any section can be split into two sections, the split command takes arguments
which handle how it should be split, including which side of the split the
current content of the top panel should be saved to.

Splits can be either horizontal or vertical, and can occur between any two
columns/rows within that section.

Only the top panel of a section is split, if that panel is popped off, whatever
was underneath it will unchanged by the split.

A panel containing a split can be split: the current split will be saved to
one of the subsections created. This includes the base panel of the entire
screen.

# Commands that can be applied to split sections

These commands can be applied to sections which are split. Applying them to
sections which are not split produces no change.

## Unsplitting a Section

A split section can be unsplit. The unsplit command takes an argument
identifying which half of the split should be saved. The stack from the saved
section will be pushed on top of the stack in the section that is being
unsplit.

## Adjusting a Section Split

The division within a split section can be adjusted, changing how that section
is split. This adjustment can change both the position and axis of the split,
so that a horizontal split could become a vertical split, for example. The
contents of the two subsections of the split will be resized to fit the new
areas of the subsections.

# The 'active' section

At any given time, exactly one section of the screen is marked the active
section. This top panel of this section must be a character grid.

All commands which apply to character grids - writing characters, setting
styles, moving the cursor, and so on - are applied to the grid which is
currently active.

A command exists to switch which section is active at a given time. An attempt
to switch the active section to a section which does not have a character grid
as its top panel will result in no change.
