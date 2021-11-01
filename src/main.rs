use conways::{Cell, Output};

use draw::Draw;

use std::convert::TryInto;
use std::io::{stdout, Write};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Instant;

use std::time::Duration;
use termion::{clear, color, cursor, event::Key, input::TermRead, raw::IntoRawMode};

macro_rules! puts {
    ($raw: expr, $e:expr $(,)?) => { { let mut lock = $raw.lock().unwrap(); write!(lock, "{}", $e).unwrap(); lock.flush().unwrap() } };
    ($raw: expr,$e:expr, $($rest: expr),+) => { { puts!($raw, $e); $(puts!($raw, $rest));+ } }
}

fn main() {
    let raw = Arc::new(Mutex::new(stdout().into_raw_mode().unwrap()));
    let raw_clone = raw.clone();

    puts!(raw, clear::All, cursor::Goto(1, 1), cursor::SteadyUnderline);

    let (w, h) = termion::terminal_size().unwrap();
    let (w, h) = (usize::from(w), usize::from(h));

    let cells = Arc::new(Mutex::new(vec![Cell(false); w * h]));
    let not_started = Arc::new(AtomicBool::new(true));
    let ended = Arc::new(AtomicBool::new(false));
    let found = Arc::new(AtomicBool::new(false));
    let waiting = Arc::new(AtomicBool::new(false));

    let not_started_clone = not_started.clone();
    let ended_clone = ended.clone();
    let cells_clone = cells.clone();
    let found_clone = found.clone();
    let waiting_clone = waiting.clone();

    thread::spawn(move || {
        let mut keys = std::io::stdin().keys();
        let mut x: usize = 0;
        let mut y: usize = 0;
        let mut output = Output::new();

        loop {
            let k = keys.next().unwrap().unwrap();

            if not_started_clone.load(Ordering::SeqCst) {
                match k {
                    Key::Char('x') => {
                        let mut borrowed = cells_clone.lock().unwrap();

                        borrowed[y * w + x].oppo();
                        borrowed[y * w + x]
                            .draw(&mut *raw_clone.lock().unwrap())
                            .unwrap();

                        puts!(raw_clone, cursor::Left(1));
                    }
                    Key::Char('s') => {
                        puts!(raw_clone, cursor::Hide);

                        x = 0;
                        y = 0;

                        not_started_clone.store(false, Ordering::SeqCst);
                    }
                    Key::Up => {
                        y = y.saturating_sub(1);
                    }
                    Key::Down => {
                        if y < h - 1 {
                            y += 1;
                        }
                    }
                    Key::Right => {
                        if x < w - 1 {
                            x += 1;
                        }
                    }
                    Key::Left => {
                        x = x.saturating_sub(1);
                    }
                    _ => {}
                }

                puts!(
                    raw_clone,
                    cursor::Goto((x + 1).try_into().unwrap(), (y + 1).try_into().unwrap())
                );
            } else if let Key::Char('e') = k {
                not_started_clone.store(true, Ordering::SeqCst);
                waiting_clone.store(true, Ordering::SeqCst);

                while !found_clone.load(Ordering::SeqCst) {
                    std::hint::spin_loop();
                }

                found_clone.store(false, Ordering::SeqCst);
                waiting_clone.store(false, Ordering::SeqCst);

                puts!(raw_clone, cursor::Goto(1, 1));

                let lock = cells_clone.lock().unwrap();

                for i in 0..w * h {
                    let y = i / w;

                    if y * w == i && i > 0 {
                        output.write_all(b"\r\n").unwrap();
                    }
                    lock[i].draw(&mut output).unwrap();
                }

                output.flush_to(&mut *raw_clone.lock().unwrap());

                puts!(raw_clone, cursor::Goto(1, 1), cursor::Show);

                x = 0;
                y = 0;
            }

            if let Key::Char('q') = k {
                ended_clone.store(true, Ordering::SeqCst);
                return;
            }
        }
    });

    let mut output = Output::new();

    'outer: loop {
        let start = Instant::now();

        if ended.load(Ordering::SeqCst) {
            break;
        }

        let cloned = cells.lock().unwrap().clone();

        if !not_started.load(Ordering::SeqCst) {
            for i in 0..w * h {
                let y = i / w;

                if y * w == i && i > 0 {
                    output.write_all(b"\r\n").unwrap();
                }

                if not_started.load(Ordering::SeqCst) && waiting.load(Ordering::SeqCst) {
                    *cells.lock().unwrap() = cloned;

                    found.store(true, Ordering::SeqCst);

                    continue 'outer;
                }

                let mut neighbors = 0;

                for y in -1i128..=1 {
                    for x in -1i128..=1 {
                        neighbors += cloned
                            [((w * h) as i128 + i as i128 + y * w as i128 + x) as usize % (w * h)]
                            .0 as usize;
                    }
                }

                let mut lock = cells.lock().unwrap();

                if lock[i].0 {
                    neighbors -= 1;
                }

                lock[i].update(neighbors);
                lock[i].draw(&mut output).unwrap();
            }

            output.flush_to(&mut *raw.lock().unwrap());

            thread::sleep(Duration::from_millis(15).saturating_sub(Instant::now() - start));
            puts!(raw, cursor::Goto(1, 1));
        } else if waiting.load(Ordering::SeqCst) {
            found.store(true, Ordering::SeqCst);
        }
    }

    puts!(
        raw,
        clear::All,
        cursor::Restore,
        cursor::Show,
        cursor::Goto(1, 1),
        color::Bg(color::Black)
    );
}
