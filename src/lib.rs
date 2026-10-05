//! Детерминированный генератор данных по сиду.

use rand::{RngExt, SeedableRng, rngs::ChaCha8Rng};
use std::{
    fs,
    io::{self, Write},
    num::ParseIntError,
    path::Path,
};

/// Генератор, чья последовательность полностью определяется сидом.
pub struct SeededGenerator {
    rng: ChaCha8Rng,
}

impl SeededGenerator {
    /// Создаёт генератор из сида.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    /// Строка из `len` печатных ASCII-символов (`!`..=`~`).
    pub fn ascii(&mut self, len: usize) -> String {
        (0..len)
            .map(|_| char::from(self.rng.random_range(b'!'..=b'~')))
            .collect()
    }

    /// Массив из `len` случайных байт.
    pub fn bytes(&mut self, len: usize) -> Vec<u8> {
        let mut buf = vec![0u8; len];
        self.rng.fill(buf.as_mut_slice());
        buf
    }

    /// Записывает `len` случайных байт в файл `path`.
    ///
    /// # Errors
    ///
    /// Возвращает ошибку, если файл не удалось записать.
    pub fn write_bytes(&mut self, len: usize, path: &Path) -> io::Result<()> {
        fs::write(path, self.bytes(len))
    }
    /// Записывает `len` случайных байт в произвольный приёмник
    /// (stdout, сокет, буфер в памяти и т. п.).
    ///
    /// # Errors
    ///
    /// Возвращает ошибку, если запись не удалась.
    pub fn write_bytes_to<W: Write>(&mut self, len: usize, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.bytes(len))
    }
}

/// Разбирает сид из пользовательского ввода (пробелы по краям игнорируются).
///
/// # Errors
///
/// Возвращает ошибку, если ввод не является числом `u64`.
pub fn parse_seed(input: &str) -> Result<u64, ParseIntError> {
    input.trim().parse()
}

/// Разбирает длину; пустой ввод означает `default`.
///
/// # Errors
///
/// Возвращает ошибку, если ввод не является неотрицательным целым числом.
pub fn parse_len(input: &str, default: usize) -> Result<usize, ParseIntError> {
    let input = input.trim();
    if input.is_empty() {
        Ok(default)
    } else {
        input.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("uu-pseudogen-test-{}-{name}", std::process::id()))
    }

    #[test]
    fn ascii_is_deterministic() {
        let a = SeededGenerator::new(42).ascii(64);
        let b = SeededGenerator::new(42).ascii(64);
        assert_eq!(a, b);
    }

    #[test]
    fn ascii_differs_between_seeds() {
        let a = SeededGenerator::new(1).ascii(64);
        let b = SeededGenerator::new(2).ascii(64);
        assert_ne!(a, b);
    }

    #[test]
    fn ascii_has_requested_length() {
        for len in [0, 1, 32, 1000] {
            assert_eq!(SeededGenerator::new(7).ascii(len).len(), len);
        }
    }

    #[test]
    fn ascii_uses_only_printable_non_space_chars() {
        let s = SeededGenerator::new(123).ascii(10_000);
        assert!(s.bytes().all(|b| (b'!'..=b'~').contains(&b)));
    }

    #[test]
    fn ascii_shorter_output_is_prefix_of_longer() {
        let short = SeededGenerator::new(5).ascii(16);
        let long = SeededGenerator::new(5).ascii(64);
        assert!(long.starts_with(&short));
    }

    #[test]
    fn ascii_consecutive_calls_continue_the_stream() {
        let mut g = SeededGenerator::new(9);
        let first = g.ascii(16);
        let second = g.ascii(16);
        let whole = SeededGenerator::new(9).ascii(32);
        assert_eq!(format!("{first}{second}"), whole);
    }

    #[test]
    fn bytes_are_deterministic() {
        let a = SeededGenerator::new(42).bytes(256);
        let b = SeededGenerator::new(42).bytes(256);
        assert_eq!(a, b);
    }

    #[test]
    fn bytes_differ_between_seeds() {
        let a = SeededGenerator::new(1).bytes(256);
        let b = SeededGenerator::new(2).bytes(256);
        assert_ne!(a, b);
    }

    #[test]
    fn bytes_have_requested_length() {
        for len in [0, 1, 3, 4, 5, 63, 64, 65, 1000] {
            assert_eq!(SeededGenerator::new(7).bytes(len).len(), len);
        }
    }

    #[test]
    fn bytes_are_not_constant() {
        let v = SeededGenerator::new(0).bytes(256);
        assert!(v.iter().any(|&b| b != v[0]));
    }

    #[test]
    fn write_bytes_writes_same_data_as_bytes() {
        let path = temp_path("write.bin");
        SeededGenerator::new(77).write_bytes(100, &path).unwrap();
        let on_disk = fs::read(&path).unwrap();
        fs::remove_file(&path).unwrap();
        assert_eq!(on_disk, SeededGenerator::new(77).bytes(100));
    }

    #[test]
    fn write_bytes_reports_io_error() {
        let path = temp_path("no-such-dir").join("out.bin");
        assert!(SeededGenerator::new(1).write_bytes(8, &path).is_err());
    }

    #[test]
    fn parse_seed_accepts_numbers_with_whitespace() {
        assert_eq!(parse_seed(" 12345\n").unwrap(), 12345);
        assert_eq!(parse_seed("0").unwrap(), 0);
        assert_eq!(parse_seed("18446744073709551615").unwrap(), u64::MAX);
    }

    #[test]
    fn parse_seed_rejects_garbage() {
        assert!(parse_seed("").is_err());
        assert!(parse_seed("abc").is_err());
        assert!(parse_seed("-1").is_err());
        assert!(parse_seed("18446744073709551616").is_err());
    }

    #[test]
    fn parse_len_uses_default_on_empty_input() {
        assert_eq!(parse_len("", 32).unwrap(), 32);
        assert_eq!(parse_len("  \n", 32).unwrap(), 32);
    }

    #[test]
    fn parse_len_parses_numbers_and_rejects_garbage() {
        assert_eq!(parse_len("64", 32).unwrap(), 64);
        assert_eq!(parse_len("0", 32).unwrap(), 0);
        assert!(parse_len("-5", 32).is_err());
        assert!(parse_len("x", 32).is_err());
    }
    #[test]
    fn ascii_golden_value() {
        // Зафиксированный вывод для сида 20 и длины 20.
        // Если тест упал после обновления rand, изменился алгоритм
        // или формат вывода, и старые сиды больше не воспроизводятся.
        assert_eq!(SeededGenerator::new(20).ascii(20), "wolR+VZrBKkheqrX2}+w");
    }
    #[test]
    fn write_bytes_to_matches_bytes() {
        let mut out = Vec::new();
        SeededGenerator::new(77)
            .write_bytes_to(100, &mut out)
            .unwrap();
        assert_eq!(out, SeededGenerator::new(77).bytes(100));
    }

    #[test]
    fn write_bytes_to_propagates_errors() {
        struct Failing;
        impl Write for Failing {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::ErrorKind::BrokenPipe.into())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let err = SeededGenerator::new(1)
            .write_bytes_to(8, &mut Failing)
            .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::BrokenPipe);
    }
    #[test]
    fn bytes_golden_value() {
        // Зафиксированный вывод для сида 10 и длины 10.
        assert_eq!(
            SeededGenerator::new(10).bytes(10),
            [0xf7, 0x71, 0xa0, 0x56, 0xdd, 0xac, 0x53, 0x8f, 0x92, 0x14]
        );
    }
}
