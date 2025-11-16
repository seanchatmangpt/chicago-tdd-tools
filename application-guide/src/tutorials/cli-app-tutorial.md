# Building a Real CLI Application: Complete Tutorial

> 🎓 **TUTORIAL** | Build a complete CLI application using Chicago TDD

This tutorial walks you through building a real command-line todo application from scratch using Chicago TDD principles.

**Prerequisites**: [Getting Started](getting-started.md), [Fixtures Deep Dive](fixtures-tutorial.md)
**Time**: ~45 minutes
**What you'll build**: A working `todo-cli` application with tests

---

## Project Overview

You'll build a **todo-cli** app with these features:

```bash
# Add a todo
$ todo-cli add "Buy groceries"

# List todos
$ todo-cli list

# Complete a todo
$ todo-cli done 1

# Delete a todo
$ todo-cli delete 1
```

This is a real, testable application.

---

## Step 1: Project Setup (2 minutes)

### Create the project

```bash
cargo new todo-cli
cd todo-cli
```

### Add dependencies

Edit `Cargo.toml`:

```toml
[package]
name = "todo-cli"
version = "0.1.0"
edition = "2021"

[dependencies]
# (no dependencies for basic version)

[dev-dependencies]
chicago-tdd-tools = { version = "1.1", features = ["testing-extras"] }
```

### Project structure

```
todo-cli/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── add.rs
│   │   ├── list.rs
│   │   ├── done.rs
│   │   └── delete.rs
│   └── store.rs          # Todo storage
└── tests/
    ├── cli_tests.rs
    └── commands_tests.rs
```

---

## Step 2: Core Data Structures (5 minutes)

### Define a Todo struct

Create `src/store.rs`:

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct Todo {
    pub id: u32,
    pub title: String,
    pub completed: bool,
}

#[derive(Clone, Debug)]
pub struct TodoStore {
    todos: Vec<Todo>,
    next_id: u32,
}

impl TodoStore {
    pub fn new() -> Self {
        TodoStore {
            todos: Vec::new(),
            next_id: 1,
        }
    }

    pub fn add(&mut self, title: &str) -> u32 {
        let id = self.next_id;
        self.todos.push(Todo {
            id,
            title: title.to_string(),
            completed: false,
        });
        self.next_id += 1;
        id
    }

    pub fn list(&self) -> Vec<&Todo> {
        self.todos.iter().collect()
    }

    pub fn mark_done(&mut self, id: u32) -> bool {
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
            todo.completed = true;
            true
        } else {
            false
        }
    }

    pub fn delete(&mut self, id: u32) -> bool {
        let original_len = self.todos.len();
        self.todos.retain(|t| t.id != id);
        self.todos.len() < original_len
    }
}
```

---

## Step 3: Test the Core Logic (8 minutes)

Create `tests/commands_tests.rs`:

```rust
use chicago_tdd_tools::prelude::*;
use todo_cli::store::{Todo, TodoStore};

test!(test_add_todo, {
    let mut store = TodoStore::new();

    let id = store.add("Buy groceries");

    assert_eq!(id, 1);
    let todos = store.list();
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].title, "Buy groceries");
    assert!(!todos[0].completed);
});

test!(test_add_multiple, {
    let mut store = TodoStore::new();

    let id1 = store.add("Task 1");
    let id2 = store.add("Task 2");
    let id3 = store.add("Task 3");

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);

    assert_eq!(store.list().len(), 3);
});

test!(test_mark_done, {
    let mut store = TodoStore::new();
    let id = store.add("Task");

    let result = store.mark_done(id);
    assert!(result);

    let todos = store.list();
    assert!(todos[0].completed);
});

test!(test_mark_nonexistent_as_done, {
    let mut store = TodoStore::new();

    let result = store.mark_done(999);

    assert!(!result);
});

test!(test_delete_todo, {
    let mut store = TodoStore::new();
    store.add("Task 1");
    store.add("Task 2");

    let result = store.delete(1);

    assert!(result);
    assert_eq!(store.list().len(), 1);
});

test!(test_delete_nonexistent, {
    let mut store = TodoStore::new();

    let result = store.delete(999);

    assert!(!result);
});

test!(test_empty_store, {
    let store = TodoStore::new();

    assert_eq!(store.list().len(), 0);
});
```

Run tests: `cargo test`

---

## Step 4: CLI Commands (10 minutes)

Create `src/commands/add.rs`:

```rust
use crate::store::TodoStore;

pub fn execute(store: &mut TodoStore, args: &[String]) -> Result<String, String> {
    if args.is_empty() {
        return Err("Usage: add <title>".to_string());
    }

    let title = args.join(" ");
    let id = store.add(&title);

    Ok(format!("Added todo #{}: {}", id, title))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chicago_tdd_tools::prelude::*;

    test!(test_add_command, {
        let mut store = TodoStore::new();
        let args = vec!["Buy milk".to_string()];

        let result = execute(&mut store, &args);

        assert_ok!(&result);
        assert!(result.unwrap().contains("Buy milk"));
        assert_eq!(store.list().len(), 1);
    });

    test!(test_add_with_spaces, {
        let mut store = TodoStore::new();
        let args = vec!["Buy".to_string(), "milk".to_string(), "and".to_string(), "eggs".to_string()];

        let result = execute(&mut store, &args);

        assert_ok!(&result);
        let msg = result.unwrap();
        assert!(msg.contains("Buy milk and eggs"));
    });

    test!(test_add_no_args, {
        let mut store = TodoStore::new();
        let args = vec![];

        let result = execute(&mut store, &args);

        assert_err!(&result);
    });
}
```

Create `src/commands/list.rs`:

```rust
use crate::store::TodoStore;

pub fn execute(store: &TodoStore) -> String {
    let todos = store.list();

    if todos.is_empty() {
        return "No todos".to_string();
    }

    let mut output = String::new();
    for todo in todos {
        let status = if todo.completed { "✓" } else { " " };
        output.push_str(&format!("[{}] #{}: {}\n", status, todo.id, todo.title));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use chicago_tdd_tools::prelude::*;

    test!(test_list_empty, {
        let store = TodoStore::new();

        let output = execute(&store);

        assert_eq!(output, "No todos");
    });

    test!(test_list_with_items, {
        let mut store = TodoStore::new();
        store.add("Task 1");
        store.add("Task 2");

        let output = execute(&store);

        assert!(output.contains("Task 1"));
        assert!(output.contains("Task 2"));
    });

    test!(test_list_shows_completion, {
        let mut store = TodoStore::new();
        let id = store.add("Task");
        store.mark_done(id);

        let output = execute(&store);

        assert!(output.contains("✓"));
    });
}
```

---

## Step 5: Main Entry Point (5 minutes)

Create `src/main.rs`:

```rust
use std::env;
use std::io::{self, BufRead};

mod commands;
mod store;

use commands::{add, list, done, delete};
use store::TodoStore;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = TodoStore::new();

    let stdin = io::stdin();
    let reader = stdin.lock();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.trim().split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        let command = parts[0];
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

        let result = match command {
            "add" => add::execute(&mut store, &args),
            "list" => Ok(list::execute(&store)),
            "done" => done::execute(&mut store, &args),
            "delete" => delete::execute(&mut store, &args),
            _ => Err(format!("Unknown command: {}", command)),
        };

        match result {
            Ok(msg) => println!("{}", msg),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}
```

---

## Step 6: Integration Tests (10 minutes)

Create `tests/cli_tests.rs`:

```rust
use chicago_tdd_tools::prelude::*;
use todo_cli::store::TodoStore;

test!(complete_workflow, {
    let mut store = TodoStore::new();

    // Add some todos
    store.add("Buy groceries");
    store.add("Pay bills");
    store.add("Call mom");

    let todos = store.list();
    assert_eq!(todos.len(), 3);

    // Mark one as done
    store.mark_done(2);

    // Delete one
    store.delete(3);

    let final_todos = store.list();
    assert_eq!(final_todos.len(), 2);
    assert!(!final_todos[0].completed);
    assert!(final_todos[1].completed);
});

test!(id_increment, {
    let mut store = TodoStore::new();

    let id1 = store.add("First");
    let id2 = store.add("Second");
    let id3 = store.add("Third");

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);

    store.delete(id2);  // Delete middle one

    let id4 = store.add("Fourth");
    assert_eq!(id4, 4);  // ID still increments
});
```

---

## Step 7: Testing with Fixtures (5 minutes)

Create `tests/fixture_tests.rs`:

```rust
use chicago_tdd_tools::prelude::*;
use chicago_tdd_tools::fixture::*;
use todo_cli::store::TodoStore;
use std::collections::HashMap;

test!(fixture_based_workflow, {
    let fixture = TestFixture::new()?;
    let mut store = TodoStore::new();

    // Phase 1: Initial setup
    let id1 = store.add("Task 1");
    let id2 = store.add("Task 2");
    fixture.set_metadata("initial_count", "2");
    fixture.capture_snapshot(HashMap::from([
        ("phase".to_string(), "1".to_string()),
        ("todos".to_string(), "2".to_string()),
    ]));

    // Phase 2: Mark complete
    store.mark_done(id1);
    fixture.set_metadata("current_phase", "mark_done");
    fixture.capture_snapshot(HashMap::from([
        ("phase".to_string(), "2".to_string()),
        ("completed".to_string(), "1".to_string()),
    ]));

    // Phase 3: Delete
    store.delete(id2);
    fixture.set_metadata("current_phase", "delete");
    fixture.capture_snapshot(HashMap::from([
        ("phase".to_string(), "3".to_string()),
        ("remaining".to_string(), "1".to_string()),
    ]));

    // Verify
    assert_eq!(fixture.snapshots().len(), 3);
    assert_eq!(store.list().len(), 1);
});
```

---

## Step 8: Running Everything (5 minutes)

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --lib

# Run integration tests only
cargo test --test '*'

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_add_todo
```

Expected output:
```
running X tests
...
test result: ok. X passed; 0 failed; 0 ignored; X measured
```

---

## Summary

You've built a real CLI application with:

✅ Core data structures with tests
✅ Command modules with unit tests
✅ Integration tests
✅ Fixture-based tests
✅ Main CLI entry point
✅ Error handling
✅ Real-world usage patterns

## Next Steps

**Enhance your application:**
- Add persistence (save todos to file)
- Add priorities to todos
- Add due dates
- Add categories/tags
- Build a web API version

**Learn more:**
- [Snapshot Testing](../advanced/snapshot-testing.md) - Test CLI output
- [Advanced Techniques](../advanced/README.md) - More sophisticated testing
- [Best Practices](../guides/best-practices.md) - Production patterns

**Share your code:**
- Push to GitHub
- Add CI/CD pipeline
- Write documentation
- Create GitHub issues for features

---

**Congratulations!** You've built and tested a real CLI application using Chicago TDD. You're now ready to build production applications!
