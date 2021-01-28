use super::{
    repl::Repl,
    Error,
};
use remap::{state, Formatter, Object, Program, Runtime, Value};
use remap_functions::{all as funcs, map};
use rustyline::{error::ReadlineError, Editor};

struct Tutorial {
    number: &'static str, // Making this a string allows for 1.1, 2.5, etc.
    title: &'static str,
    help_text: &'static str,
    correct_answer: Value,
    object: Value,
}

pub fn resolve(
    object: &mut impl Object,
    runtime: &mut Runtime,
    program: &str,
    state: &mut state::Compiler,
) -> Result<Value, Error> {
    let program = match Program::new_with_state(program.to_owned(), &funcs(), None, true, state) {
        Ok((program, _)) => program,
        Err(diagnostics) => {
            let msg = Formatter::new(program, diagnostics).colored().to_string();
            return Err(Error::Parse(msg))
        }
    };

    match runtime.run(object, &program) {
        Ok(v) => Ok(v),
        Err(err) => Err(Error::Runtime(err.to_string()))
    }
}

pub fn tutorial() -> Result<(), Error> {
    let mut index = 0;
    let mut compiler_state = state::Compiler::default();
    let mut rt = Runtime::new(state::Program::default());
    let mut rl = Editor::<Repl>::new();
    rl.set_helper(Some(Repl::new()));

    let assignment_tut = Tutorial {
        number: "1.1",
        title: "Assigning values",
        help_text: ASSIGNMENT_TEXT,
        correct_answer: Value::from(map!["severity": "info"]),
        object: Value::from(map![]),
    };

    let mut tutorials = vec![assignment_tut];

    println!("\nWelcome to the Vector Remap Language interactive tutorial!\n");

    print_tutorial_help_text(index, &tutorials);

    loop {
        let readline = rl.readline("$ ");
        match readline.as_deref() {
            Ok(line) if line == "exit" || line == "quit" => break,
            Ok(line) => {
                rl.add_history_entry(line);

                match line {
                    "help" => help(),
                    "next" => {
                        if (index + 1) == tutorials.len() {
                            println!(
                                "You've finished all the steps! Taking you back to the beginning\n"
                            );
                            index = 0;
                        } else {
                            index = index.saturating_add(1);
                        }

                        print_tutorial_help_text(index, &tutorials);
                    }
                    "prev" => {
                        if index == 0 {
                            println!("\n\nYou're back at the beginning!\n\n");
                        }

                        index = index.saturating_sub(1);
                        print_tutorial_help_text(index, &tutorials);
                    }
                    "" => continue,
                    command => {
                        let tut = &mut tutorials[index];
                        let object = &mut tut.object;
                        match resolve(object, &mut rt, command, &mut compiler_state) {
                            Ok(result) => {
                                if object == &tut.correct_answer {
                                    println!("\n\nThat's correct!\n");
                                    println!("{}", object);

                                    if (index + 1) == tutorials.len() {
                                        println!("\n\nCongratulations! You've successfully completed the VRL tutorial.");
                                        break;
                                    } else {
                                        println!("\n\nMoving on to the next exercise...");
                                        index = index.saturating_sub(1);
                                        print_tutorial_help_text(index, &tutorials);
                                    }
                                } else {
                                    println!("{}", result);
                                }
                            },
                            Err(err) => {
                                println!("{}", err);
                            },
                        }
                    }
                };
            }
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("unable to read line: {}", err);
                break;
            }
        }
    }

    Ok(())
}

fn help() {
    println!("{}", HELP_TEXT);
}

fn print_tutorial_help_text(index: usize, tutorials: &[Tutorial]) {
    let tut = &tutorials[index];

    if index != 0 {
        println!("------------");
    }

    println!(
        "Tutorial {}: {}\n\n{}\nInitial event object:\n{}\n",
        tut.number, tut.title, tut.help_text, tut.object
    );
}

// Help text
const HELP_TEXT: &str = r#"
Tutorial commands:
  next     Load the next tutorial
  prev     Load the previous tutorial
  exit     Exit the VRL interactive tutorial
"#;

const ASSIGNMENT_TEXT: &str = r#"In VRL, you can assign values to fields like this:

.field = "value"

TASK:
Assign the string `"info"` to the field `severity`.
"#;
