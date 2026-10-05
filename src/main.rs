use clap::{Parser, Subcommand};
use rpassword::prompt_password;
use std::{
    error::Error,
    io::{ErrorKind, Write, stderr, stdin, stdout},
    path::PathBuf,
};
use uu_pseudogen::{SeededGenerator, parse_len, parse_seed};

const DEFAULT_LEN: usize = 32;

#[derive(Parser)]
#[command(version, about = "Детерминированный генератор по сиду")]
struct Cli {
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Subcommand)]
enum Mode {
    /// Вывести ASCII-строку (печатные символы `!`..=`~`)
    Ascii,
    /// Сохранить массив случайных байт в файл
    Bytes {
        /// Путь к выходному файлу
        #[arg(short, long)]
        output: PathBuf,
    },
}

fn ask_len() -> Result<usize, Box<dyn Error>> {
    eprint!("Length [{DEFAULT_LEN}]: ");
    stderr().flush()?;

    let mut line = String::new();
    stdin().read_line(&mut line)?;

    parse_len(&line, DEFAULT_LEN)
        .map_err(|_| "длина должна быть неотрицательным целым числом".into())
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let len = ask_len()?;

    let input = prompt_password("Seed: ")?;
    let seed = parse_seed(&input).map_err(|_| "сид должен быть целым числом (u64)")?;

    let mut generator = SeededGenerator::new(seed);

    match cli.mode {
        Mode::Ascii => println!("{}", generator.ascii(len)),
        Mode::Bytes { output } => {
            if output.as_os_str() == "-" {
                let mut out = stdout().lock();
                match generator
                    .write_bytes_to(len, &mut out)
                    .and_then(|()| out.flush())
                {
                    // Читатель закрыл pipe раньше (например, `| head -c 4`): не ошибка.
                    Err(e) if e.kind() == ErrorKind::BrokenPipe => {}
                    result => result?,
                }
            } else {
                generator.write_bytes(len, &output)?;
                eprintln!("записано {len} байт в {}", output.display());
            }
        }
    }

    Ok(())
}
