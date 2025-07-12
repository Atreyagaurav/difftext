Get difference from latex files into a typst format.

I made this for pursonal purpose, so if you want to use it, modify it to your use case and use it.

It should work with other file formats as well, as long as you are only generating diffs, you'll have to manually fix the syntax that are different in them.

Add this on your typst file:

```typ
#let add(new) = text(blue, new)
#let rem(old) = strike(text(red, old))
#let diff(old, new) = {
    rem(old)
    add(new)
}
```

Then just use the program to generate diffs.

There are 3 modes of use:
1. Interactive mode (no args), run and provide old and new text to generate diffs in a loop. Press `Enter` then `Ctrl+D` to end a input.
2. FileDiff mode: You can provide two version of latex files as input, then provide the label to get the diff of that paragraph. The label in LaTeX should be in this format: `\paralable{par:<p>}` replace `<p>` with unique label for each paragraph.

	Use the following code in LaTeX preamble for the labels, You can also set a dummy command to do nothing.
```latex
\newcounter{para}
\setcounter{para}{1}
\newcommand{\paralabel}[1]{[\thepara]\stepcounter{para}}
```
3. FileDiff with BibTex. Provide `.aux` file for the latex to extract bibtex information and replace `\citep` and `\cite` commands with author-year citations.


Mode 1 should work for any inputs. But mode 2 and 3 are made for latex. You can modify the code for mode 2 to use a different paragraph identifier to use in other syntax. I am using a paragraph label instead of doing whole text diff so that it works even when paragraph are moved, and I need less processing power.
