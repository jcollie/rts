// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

extern crate scoped_threadpool;
extern crate chrono;

use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::io;
use std::io::Write;
use std::io::Read;
use std::vec::Vec;
use std::env;
use chrono::Utc;


use scoped_threadpool::Pool;

fn outputter(marker: &[u8; 1], output: Arc<Mutex<dyn Write>>, input: &mut dyn Read) {
    let mut vec = Vec::new();
    let mut start = Utc::now();

    loop {
        let mut buf = [0; 1];
        let rsize = input.read(&mut buf).expect("can't read stdout");
        if rsize == 0 { break; }
        if vec.len() == 0 {
            start = Utc::now();
        }
        vec.push(buf[0]);
        if buf[0] == 10 {
            {
                let mut output = output.lock().unwrap();
                output.write(marker).unwrap();
                output.write(b" ").unwrap();
                output.write(start.to_rfc3339().as_bytes()).unwrap();
                output.write(b" ").unwrap();
                output.write(vec.as_slice()).unwrap();
                output.flush().unwrap();
            }
            vec = Vec::new();
        }
    }
}

fn main() {
    let mut args: Vec<String> = env::args().collect();

    args.remove(0);

    let mut cmd =
        Command::new(args.remove(0))
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn");

    let stdout = cmd.stdout.as_mut().unwrap();
    let stderr = cmd.stderr.as_mut().unwrap();
    let mut pool = Pool::new(2);
    let output = Arc::new(Mutex::new(io::stderr()));

    pool.scoped(|scope| {
        let output_stdout = output.clone();
        scope.execute(move || {
            outputter(b"O", output_stdout, stdout);
        });
        let output_stderr = output.clone();
        scope.execute(move || {
            outputter(b"E", output_stderr, stderr);
        });
    });


    cmd.wait().unwrap();
}
