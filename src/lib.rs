use std::collections::HashMap;

/*
<message>  ::= [ <prefix> <SPACE> ] <command> <params> <crlf>
<prefix>   ::= <servername> | <nick> [ '!' <user> ] [ '@' <host> ]
<command>  ::= <letter> { <letter> } | <number> <number> <number>
<SPACE>    ::= ' ' { ' ' }
<params>   ::= <SPACE> [ ':' <trailing> | <middle> <params> ]

<middle>   ::= <Any *non-empty* sequence of octets not including SPACE
               or NUL or CR or LF, the first of which may not be ':'>
<trailing> ::= <Any, possibly *empty*, sequence of octets not including
                 NUL or CR or LF>

<crlf>     ::= CR LF
*/

#[derive(Debug)]
pub struct Message {
    tags: Option<HashMap<String, Option<String>>>,
    prefix: Option<String>,
    command: Option<String>,
    params: Option<Vec<String>>,
}

impl Message {
    pub fn parse(message: String) -> Message {
        let mut msg = Message {
            tags: None,
            prefix: None,
            command: None,
            params: None,
        };

        let split_message: Vec<String> = message.split_whitespace().map(String::from).collect();
        let mut _loc: usize = 0;

        // tags
        if split_message[_loc].chars().next().unwrap() == '@' {
            let tags: HashMap<String, Option<String>> = split_message[_loc][1..]
                .split(';')
                .map(|kv| kv.split('='))
                .map(|mut kv| {
                    let k: String = kv.next().unwrap().into();
                    let mut v: Option<String> = Some(kv.next().unwrap().into());
                    if v == Some("".to_string()) {
                        v = None
                    };

                    (k, v)
                })
                .collect();

            msg.tags = Some(tags);
            _loc += 1;
        }

        // prefix
        if split_message[_loc].chars().next().unwrap() == ':' {
            msg.prefix = Some(split_message[_loc].to_owned());
            _loc += 1;
        }

        // command
        msg.command = Some(split_message[_loc].to_owned());

        // params
        _loc += 1;

        if _loc >= split_message.len() - 1 {
            return msg;
        }

        let mut params = Vec::new();

        while _loc < split_message.len() && split_message[_loc][..1] != ":".to_string() {
            params.push(split_message[_loc].to_owned());
            _loc += 1;
        }

        params.push(split_message[_loc..split_message.len()].join(" ")[1..].to_string());
        msg.params = Some(params);

        msg
    }
}
