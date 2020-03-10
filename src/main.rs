// This file is part of rts.
//
// rts is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// rts is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this rts.  If not, see <https://www.gnu.org/licenses/>.

extern crate scoped_threadpool;
extern crate chrono;

use std::process::{Command, Stdio};
use std::io::{self, Read, Write};
use std::vec::Vec;
use std::env;
use chrono::Utc;
use scoped_threadpool::Pool;

fn outputter(marker: &[u8; 1], input: &mut dyn Read) {
    let output = io::stderr();
    let mut vec = Vec::new();
    let mut start = Utc::now();

    loop {
        let mut buf = [0; 1];
        let rsize = input.read(&mut buf).expect("can't read stdout");
        if rsize == 0 {
            if vec.len() != 0 {
                let mut handle = output.lock();
                handle.write(marker).unwrap();
                handle.write(b" ").unwrap();
                handle.write(start.to_rfc3339().as_bytes()).unwrap();
                handle.write(b" ").unwrap();
                handle.write(vec.as_slice()).unwrap();
                handle.write(b"\n").unwrap();
                handle.flush().unwrap();
            }
            break;
        }
        if vec.len() == 0 {
            start = Utc::now();
        }
        vec.push(buf[0]);
        if buf[0] == 10 {
            {
                let mut handle = output.lock();
                handle.write(marker).unwrap();
                handle.write(b" ").unwrap();
                handle.write(start.to_rfc3339().as_bytes()).unwrap();
                handle.write(b" ").unwrap();
                handle.write(vec.as_slice()).unwrap();
                handle.flush().unwrap();
            }
            vec.truncate(0);
        }
    }
}

fn main() {
    let mut args: Vec<String> = env::args().collect();

    args.remove(0);

    let mut child =
        Command::new(args.remove(0))
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn command");

    let child_stdout = child.stdout.as_mut().unwrap();
    let child_stderr = child.stderr.as_mut().unwrap();
    let mut pool = Pool::new(2);

    pool.scoped(|scope| {
        scope.execute(move || {
            outputter(b"O", child_stdout);
        });
        scope.execute(move || {
            outputter(b"E", child_stderr);
        });
    });

    child.wait().unwrap();
}
