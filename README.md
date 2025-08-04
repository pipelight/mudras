# Mudras: a jonin hotkey daemon.

Currently in Alpha.

## Motivations

I need a hotkey daemon for niri, hyprland, bspwm... (wayland and X11),
that gives me enough rope to shoot myself in the foot.

## Configuration

Very similar to niri bind configuration in **kdl**.

```kdl
Super+Enter {
  @press {
    - "kitty -e fish"
    - r#"
      notify-send "new term"
      "#
  }
}

Super {
  @release {
    // enter a submap
    @enter "niri"
  }
}


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

  Super {
    @press {
     // Exit the submap
      @exit;
    }
  }
  Escape {
    @press {
      @exit;
    }
  }
}

```

Everything in here has been stolen from
[niri](https://github.com/YaLTeR/niri)
and
[swhkd]()
