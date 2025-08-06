# Mudras: A hotkey daemon for elite shinobi.

Currently in Alpha.

A keyboard utility.
It executes commands when a key combination is matched.
Set up binds and subbinds with actions on key press and release.

_I need a hotkey daemon for niri, hyprland, bspwm... (wayland and X11),
that gives me enough rope to shoot myself in the foot._

## Configuration

A single configuration file in **kdl** at `~/.config/mudras/config.kdl`
Very similar to niri bind.

### Set up a bind (or hotkey, or binding).

- Set up a binding as a sequence of keys separated by `+`.

```kdl
Super+Enter
```

- You can attach commands on key **press** and/or key **release**.

```kdl
Super+Enter {
  @press
  @release
}
```

- Set up a list of commands to execute when binding is matched.
  Commands can be single line strings escaped with \"\"
  or multiline strings escaped with r#\"\"#.

```kdl
Super+Enter {
  @press {
    - "kitty -e fish"
    - r#"
      notify-send "new term"
      "#
  }
}
```

### Set up a submap (or mode, or subbind).

- The submap is just a named container that contains binds as defined in the upper section.

```kdl
// Submap definition
@submap name="niri" {
  Ctrl+m {
    @press {
      - "niri msg move-column-left"
    }
  }
  Ctrl+i {
    @press {
      - "niri msg move-column-right"
    }
  }
}

```

- Enter the submap.
  You enter a submap with the special command prefix `@enter`,
  followed by the **submap name**.

```kdl
Super {
  @release {
    // enter a submap
    @enter "niri";
  }
}
```

- Exit the submap.
  You exit a submap with the special command prefix `@exit`

  In the following example we exit the submap with the same
  key we use to enter (Super).

```kdl
// Submap definition
@submap name="niri" {
  Super {
    @release {
     // Exit the submap
      @exit;
    }
  }
}

```

### Ignore some bind on multiple key release (bug fix).

When defining binds like `Super+Alt+T @release `, `Super+Alt @release` and `Super @release`.
If you type `Super+Alt+T` and release keys, actions corresponding to every bind are executed.

You may want to flag the shortest bindings with `@release backward=false`
Only the longest bind is triggered, therefore `Super+Alt+T` is executed
and `Super+Alt` as of `Super` are ignored.

```kdl
Super+Alt {
  @release backward=false {
    - r#"notify-send "test" "#
  }
}
Super+Alt+T {
  @release {
    - r#"notify-send "test" "#
  }
}
```

## Install

### Cargo

```sh
cargo install --git https://github.com/pipelight/mudras
```

### Nixos with flakes

```nix
# flake.nix
inputs = {
  mudras = {
      url = "github:pipelight/mudras";
  };
};
```

```nix
# default.nix
environment.systemPackages = with pkgs; [
  inputs.mudras.packages.${system}.default;
];

```

Start with your favorite init script or window manager.

# Alternatives

Everything in here has been stolen from
[niri](https://github.com/YaLTeR/niri)
and
[swhkd](https://github.com/waycrate/swhkd)

# Developers

```sh
cargo build --release
sudo RUST_LOG=debug ./target/release/mudras

```
