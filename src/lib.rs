use std::collections::HashMap;

/*
<message>       ::= ['@' <tags> <SPACE>] [':' <prefix> <SPACE> ] <command> <params> <crlf>
<tags>          ::= <tag> [';' <tag>]*
<tag>           ::= <key> ['=' <escaped_value>]
<key>           ::= [ <client_prefix> ] [ <vendor> '/' ] <key_name>
<client_prefix> ::= '+'
<key_name>      ::= <non-empty sequence of ascii letters, digits, hyphens ('-')>
<escaped_value> ::= <sequence of zero or more utf8 characters except NUL, CR, LF, semicolon (`;`) and SPACE>
<vendor>        ::= <host>

<prefix>        ::= <servername> | <nick> [ '!' <user> ] [ '@' <host> ]
<command>       ::= <letter> { <letter> } | <number> <number> <number>
<SPACE>         ::= ' ' { ' ' }
<params>        ::= <SPACE> [ ':' <trailing> | <middle> <params> ]

<middle>        ::= <Any *non-empty* sequence of octets not including SPACE
                    or NUL or CR or LF, the first of which may not be ':'>
<trailing>      ::= <Any, possibly *empty*, sequence of octets not including
                    NUL or CR or LF>

<crlf>          ::= CR LF
*/

#[derive(Debug, PartialEq)]
pub enum Prefix<'a> {
    Servername(&'a str),
    Nick(&'a str, &'a str, &'a str),
}

#[derive(Debug)]
pub struct Message<'a> {
    pub tags: Option<HashMap<&'a str, Option<&'a str>>>,
    pub prefix: Option<Prefix<'a>>,
    pub command: Option<&'a str>,
    pub params: Option<Vec<&'a str>>,
}

impl<'a> Message<'a> {
    pub fn parse(message: &str) -> Result<Message, &'static str> {
        if &message.len() == &0 {
            return Err("Nothing found to parse");
        }

        let mut msg = Message {
            tags: None,
            prefix: None,
            command: None,
            params: None,
        };

        let message = message.trim();
        let mut tags: Option<&str> = None;
        let mut prefix: Option<&str> = None;
        let mut command: Option<&str> = None;

        match &message[..1] {
            "@" => {
                if let Some(i) = message.find(' ') {
                    tags = Some(&message[..i]);
                    if &message[i + 1..i + 2] == ":" {
                        if let Some(j) = message[i + 1..].find(' ') {
                            prefix = Some(&message[i + 1..=i + j]);
                            command = Some(&message[i + j + 2..]);
                        }
                    } else {
                        command = Some(&message[i + 1..]);
                    }
                }
            }
            ":" => {
                if let Some(i) = message.find(' ') {
                    prefix = Some(&message[..i]);
                    command = Some(&message[i + 1..]);
                }
            }
            _ => {
                command = Some(&message[..]);
            }
        }

        if let Some(d) = tags {
            msg.tags = Some(
                d[1..]
                    .split(';')
                    .map(|kv| kv.split('='))
                    .map(|mut kv| {
                        let k: &str = kv.next().unwrap();
                        let mut v: Option<&str> = Some(kv.next().unwrap());
                        if v == Some("") {
                            v = None
                        };

                        (k, v)
                    })
                    .collect(),
            );
        }

        if let Some(d) = prefix {
            let prefix: Vec<&str> = d[1..].split(|ch| ch == '!' || ch == '@').collect();

            if prefix.len() == 1 {
                msg.prefix = Some(Prefix::Servername(&prefix[0]));
            } else if prefix.len() == 3 {
                msg.prefix = Some(Prefix::Nick(&prefix[0], &prefix[1], &prefix[2]));
            }
        }

        if let Some(d) = command {
            if let Some(i) = d.find(' ') {
                msg.command = Some(&d[..i]);

                let params_string: &str = &d[i + 1..];
                let text_loc = params_string.find(':');
                let mut params: Vec<&str> = Vec::new();

                match text_loc {
                    Some(0) => {
                        params.push(&params_string[1..]);
                    }
                    Some(loc) => {
                        params = params_string[..loc - 1].split_ascii_whitespace().collect();
                        params.push(&params_string[loc + 1..]);
                    }
                    None => {
                        params = params_string.split_ascii_whitespace().collect();
                    }
                }

                msg.params = Some(params);
            } else {
                msg.command = command;
            }
        }

        Ok(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::{Message, Prefix};

    #[test]
    fn normal_message() {
        let parsed = Message::parse("@badge-info=;badges=broadcaster/1;color=#008000;display-name=715209;emotes=;flags=;id=8a90aa05-eea3-4699-84eb-1d4c65b85f94;mod=0;room-id=21621987;subscriber=0;tmi-sent-ts=1559891010190;turbo=0;user-id=21621987;user-type= :715209!715209@715209.tmi.twitch.tv PRIVMSG #715209 :hello").unwrap();

        assert_ne!(parsed.tags, None);
        assert_eq!(
            parsed.prefix,
            Some(Prefix::Nick("715209", "715209", "715209.tmi.twitch.tv"))
        );
        assert_eq!(parsed.command, Some("PRIVMSG"));
        assert_eq!(parsed.params, Some(vec!["#715209", "hello"]));
    }

    #[test]
    fn normal_message_no_tags() {
        let parsed =
            Message::parse(":715209!715209@715209.tmi.twitch.tv PRIVMSG #715209 :hello").unwrap();

        assert_eq!(parsed.tags, None);
        assert_eq!(
            parsed.prefix,
            Some(Prefix::Nick("715209", "715209", "715209.tmi.twitch.tv"))
        );
        assert_eq!(parsed.command, Some("PRIVMSG"));
        assert_eq!(parsed.params, Some(vec!["#715209", "hello"]));
    }

    #[test]
    fn ping() {
        let parsed = Message::parse("PING :tmi.twitch.tv").unwrap();

        assert_eq!(parsed.tags, None);
        assert_eq!(parsed.prefix, None);
        assert_eq!(parsed.command, Some("PING"));
        assert_eq!(parsed.params, Some(vec!["tmi.twitch.tv"]));
    }

    #[test]
    fn no_params() {
        let parsed = Message::parse("@badge-info=;badges=;color=#008000;display-name=715209;emote-sets=0,33563,231890,300206296,300242181;user-id=21621987;user-type= :tmi.twitch.tv GLOBALUSERSTATE").unwrap();

        assert_ne!(parsed.tags, None);
        assert_eq!(parsed.prefix, Some(Prefix::Servername("tmi.twitch.tv")));
        assert_eq!(parsed.command, Some("GLOBALUSERSTATE"));
        assert_eq!(parsed.params, None);
    }
    #[test]
    fn tags_no_prefix() {
        let parsed = Message::parse("@badge-info=;badges=;color=#008000;display-name=715209;emote-sets=0,33563,231890,300206296,300242181;user-id=21621987;user-type= GLOBALUSERSTATE").unwrap();

        assert_ne!(parsed.tags, None);
        assert_eq!(parsed.prefix, None);
        assert_eq!(parsed.command, Some("GLOBALUSERSTATE"));
        assert_eq!(parsed.params, None);
    }
    #[test]
    fn tags_and_params_no_prefix() {
        let parsed = Message::parse("@badge-info=;badges=;color=#008000;display-name=715209;emote-sets=0,33563,231890,300206296,300242181;user-id=21621987;user-type= PRIVMSG #715209 :hello").unwrap();

        assert_ne!(parsed.tags, None);
        assert_eq!(parsed.prefix, None);
        assert_eq!(parsed.command, Some("PRIVMSG"));
        assert_eq!(parsed.params, Some(vec!["#715209", "hello"]));
    }
    #[test]
    fn only_command() {
        let parsed = Message::parse("PRIVMSG").unwrap();

        assert_eq!(parsed.tags, None);
        assert_eq!(parsed.prefix, None);
        assert_eq!(parsed.command, Some("PRIVMSG"));
        assert_eq!(parsed.params, None);
    }

    #[test]
    fn nothing_to_parse() {
        let parsed = Message::parse("");

        assert!(parsed.is_err(), "Nothing found to parse");
    }
}
