# Time Tracker

CLI tool for tracking time committed to a given task.

Time Tracker aims to provide a simple CLI for tracking time spent
working on a given task. This can be helpful for tracking time committed
to a given work tasks for reporting and time card purposes or just
monitoring how much you've worked on your personal project(s).

# Installation

Time Tracker requires Rust version `1.65.0` or new.

First build Time Tracker:
```
> cargo build --release
```

Then symlink or copy the binary to a location in your path:
```
> mv target/release/tt ~/bin
```

# Documentation

You can build the docs via `cargo doc`:

```
cargo doc --no-deps --open
```

# Usage

The `tt` help documentation provides a list of available commands and usage
information.

```
> tt --help
```

Start tracking working on a task with the `start` command:

```
> tt start my_task
```

When you're done working on that task use the `end` command:

```
> tt end
```

If you want to mark progress on a task in the past use the `complete`
command:

```
> tt complete my_other_task "1:52:37"
```

View a summary of your work with `summary`:

```
> tt summary
2022-11-07 Stats
-----------------------
my_other_task: 1h 52m 37s (100.00% of task total)
my_task: 0h 0m 5s (100.00% of task total)

Aggregate Stats
-----------------------
my_other_task: 1h 52m 37s (99.93% of total time)
my_task: 0h 0m 5s (0.07% of total time)
```

The task summary information breaks down task time per day and displays
additional aggregate data. The "percentage of task total" is the percentage
of the total time dedicated to that task on a given day if that task was
tracked. The "percentage of total time" specifies what percent of all
tracked time was spent on a given task.
