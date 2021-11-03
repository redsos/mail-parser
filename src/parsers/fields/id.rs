/*
 * Copyright Stalwart Labs, Minter Ltd. See the COPYING
 * file at the top-level directory of this distribution.
 *
 * Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
 * https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
 * <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
 * option. This file may not be copied, modified, or distributed
 * except according to those terms.
 */

use std::borrow::Cow;

use crate::parsers::message::MessageStream;

pub fn parse_id<'x>(stream: &mut MessageStream<'x>) -> Option<Vec<Cow<'x, str>>> {
    let mut token_start: usize = 0;
    let mut token_end: usize = 0;
    let mut is_id_part = false;
    let mut ids = Vec::new();

    let mut read_pos = stream.pos;
    let mut iter = stream.data[read_pos..].iter();

    while let Some(ch) = iter.next() {
        read_pos += 1;
        match ch {
            b'\n' => match stream.data.get(read_pos) {
                Some(b' ' | b'\t') => {
                    iter.next();
                    read_pos += 1;
                    continue;
                }
                _ => {
                    stream.pos = read_pos;
                    return if !ids.is_empty() { Some(ids) } else { None };
                }
            },
            b'<' => {
                is_id_part = true;
                continue;
            }
            b'>' => {
                is_id_part = false;
                if token_start > 0 {
                    ids.push(String::from_utf8_lossy(
                        &stream.data[token_start - 1..token_end],
                    ));
                    token_start = 0;
                } else {
                    continue;
                }
            }
            b' ' | b'\t' | b'\r' => continue,
            _ => (),
        }
        if is_id_part {
            if token_start == 0 {
                token_start = read_pos;
            }
            token_end = read_pos;
        }
    }

    stream.pos = read_pos;

    None
}

#[cfg(test)]
mod tests {
    use crate::parsers::fields::id::parse_id;
    use crate::parsers::message::MessageStream;

    #[test]
    fn parse_message_ids() {
        let inputs = [
            (
                "<1234@local.machine.example>\n",
                vec!["1234@local.machine.example"],
            ),
            (
                "<1234@local.machine.example> <3456@example.net>\n",
                vec!["1234@local.machine.example", "3456@example.net"],
            ),
            (
                "<1234@local.machine.example>\n <3456@example.net> \n",
                vec!["1234@local.machine.example", "3456@example.net"],
            ),
            (
                "<1234@local.machine.example>\n\n <3456@example.net>\n",
                vec!["1234@local.machine.example"],
            ),
            (
                "              <testabcd.1234@silly.test>  \n",
                vec!["testabcd.1234@silly.test"],
            ),
            (
                "<5678.21-Nov-1997@example.com>\n",
                vec!["5678.21-Nov-1997@example.com"],
            ),
            (
                "<1234   @   local(blah)  .machine .example>\n",
                vec!["1234   @   local(blah)  .machine .example"],
            ),
        ];

        for input in inputs {
            let str = input.0.to_string();
            assert_eq!(
                input.1,
                parse_id(&mut MessageStream::new(str.as_bytes())).unwrap(),
                "Failed to parse '{:?}'",
                input.0
            );
        }
    }
}
