# Bullshism

**The calculator that is confidently, reliably, beautifully wrong.**

Bullshism is a real calculator written in Rust. It parses your expression properly,
works out the correct answer, and then gives you a different one, always with total
confidence.

```
$ bullshism 2+2
= 5 Mathematically certain.

$ bullshism "sqrt(16) * 10"
= 4.0 Peer reviewed by my cousin.
```

It is a joke. Please don't do your taxes with it.

## Features

- A full expression parser: `+ - * / % ^ !`, brackets, implicit multiplication (`2(3+4)`, `2pi`)
- Functions: `sqrt cbrt abs sin cos tan asin acos atan ln log exp floor ceil round`
- Constants: `pi e tau phi`, plus `ans` for the last answer it gave you (so lies compound)
- Ten ways to be wrong: off-by-one, flipped sign, swapped digits, "adjusted for inflation",
  a stray extra digit, a wandering decimal point, reversed digits, a "nicer" round number,
  "close enough", and simply swapping in a funnier number
- A guarantee: it never tells the truth, and the tests check that
- Interactive mode with colour, or one-shot mode for scripts
- `--confess` shows the real answer and how it was ruined
- `--seed N` makes the lies repeatable
- No dependencies at all

## Build

You need Rust (edition 2024, so Rust 1.85 or newer).

```sh
git clone https://github.com/MysticYewXd/Bullshism.git
cd Bullshism
cargo build --release
./target/release/bullshism
```

To put it on your PATH:

```sh
cargo install --path .
```

## Usage

```
bullshism [OPTIONS] [EXPRESSION...]

  -c, --confess     Also show the real answer and how it was ruined
  -s, --seed <N>    Use a fixed seed, so the same lies come out every time
  -h, --help        Show help
  -V, --version     Show the version
```

With no expression it starts an interactive session. Type `help`, `confess`
to toggle confess mode, or `quit` (Ctrl+D also works).

```
$ bullshism
Bullshism - the calculator you can't trust. Type 'help' or 'quit'.
> 5!
= 121 Source: trust me bro.
> confess
confess mode on
> 2+3*4
= 140 No further questions.
  (really 14; decimal point wandered off)
```

The one thing it won't lie about is dividing by zero.

## Tests

```sh
cargo test
```

## Licence

MIT, see [LICENSE](LICENSE).
