//! Reading a streamed job's stdout and stderr line by line, and what cut
//! either short.

use std::io::{BufRead, BufReader, Read};
use std::sync::mpsc;
use std::thread::JoinHandle;

/// Each line a pipe yields, sent as `(stream_name, line)` until the pipe
/// ends; a read that fails ends it with the stream's name and the error.
pub(super) fn pump(
    pipe: Box<dyn Read + Send>,
    stream_name: &'static str,
    sender: mpsc::Sender<(&'static str, String)>,
) -> Result<(), String> {
    let mut reader = BufReader::new(pipe);
    let mut line = Vec::new();
    loop {
        match reader.read_until(b'\n', &mut line) {
            // Nothing read into the cleared line: the pipe has ended.
            Ok(_) if line.is_empty() => return Ok(()),
            Ok(_) => {}
            Err(error) => {
                return Err(format!("the job's {stream_name} could not be read further: {error}"))
            }
        }
        // The receiver is gone only once the stream has ended.
        if sender
            .send((stream_name, String::from_utf8_lossy(&line).into_owned()))
            .is_err()
        {
            return Ok(());
        }
        line.clear();
    }
}

/// Wait for every reader and answer, for each that did not read its pipe to
/// the end, the sentence saying why: the read error, or the panic and what
/// it said.
pub(super) fn finish(pumps: Vec<(&'static str, JoinHandle<Result<(), String>>)>) -> Vec<String> {
    let mut cut = Vec::new();
    for (stream_name, pump) in pumps {
        match pump.join() {
            Ok(Ok(())) => {}
            Ok(Err(sentence)) => cut.push(sentence),
            Err(panic) => {
                let said = match (panic.downcast_ref::<&str>(), panic.downcast_ref::<String>()) {
                    (Some(text), _) => text.to_string(),
                    (_, Some(text)) => text.clone(),
                    (None, None) => "a payload that is not text".to_string(),
                };
                cut.push(format!("the reader of the job's {stream_name} panicked: {said}"));
            }
        }
    }
    cut
}
