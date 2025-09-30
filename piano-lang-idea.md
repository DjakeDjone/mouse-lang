# Music Language Specs

```mouse

// instruments
fn piano_left(note: Note) {
  play(note)
  play(note + 7),
  play(note + 16)
}
fn lead_voice(note: String) {#
  play(note);
}

fn main() {
  var i = 0;
  var note = (C4)
  while i < 12 {
    sync {
      loudness(piano_left(note));
      lead_voice(lead_voice(note));
    }
  }
}

```

``mouse
// comments

// function calls
print("Hello, world!")

```
