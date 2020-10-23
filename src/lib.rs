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

#[derive(Debug, PartialEq, Clone)]
pub enum Prefix {
    Servername(String),
    Nick(String, String, String),
}

#[derive(Debug, Clone)]
pub struct Message {
    pub tags: Option<HashMap<String, Option<String>>>,
    pub prefix: Option<Prefix>,
    pub command: Option<String>,
    pub params: Option<Vec<String>>,
}

impl Default for Message {
    fn default() -> Self {
        Message {
            tags: None,
            prefix: None,
            command: None,
            params: None,
        }
    }
}

// TODO: Replace errors with real errors
impl Message {
    pub fn parse(message: &str) -> Result<Message, &'static str> {
        if message.is_empty() {
            return Err("Nothing found to parse");
        }

        let mut msg: Message = Message::default();
        let mut pos_head = 0;
        let mut pos_tail;

        if message.starts_with('@') {
            let tags = if let Some(i) = message.find(' ') {
                pos_tail = i;
                &message[..pos_tail]
            } else {
                return Err("No command found");
            };

            msg.tags = Some(
                tags[1..]
                    .split(';')
                    .map(|kv| kv.split('='))
                    .map(|mut kv| {
                        let k: String = kv.next().unwrap().to_owned();
                        let mut v: Option<String> = Some(kv.next().unwrap().to_owned());
                        if v == Some("".to_string()) {
                            v = None
                        };

                        (k, v)
                    })
                    .collect(),
            );

            pos_head = pos_tail + 1;
        }

        if message[pos_head..].starts_with(':') {
            let prefix = if let Some(i) = &message[pos_head..].find(' ') {
                pos_tail = pos_head + i;
                &message[pos_head..pos_tail]
            } else {
                return Err("No command found");
            };

            let prefix: Vec<&str> = prefix[1..].split(|ch| ch == '!' || ch == '@').collect();

            if prefix.len() == 1 {
                msg.prefix = Some(Prefix::Servername(prefix[0].to_owned()));
            } else if prefix.len() == 3 {
                msg.prefix = Some(Prefix::Nick(
                    prefix[0].to_owned(),
                    prefix[1].to_owned(),
                    prefix[2].to_owned(),
                ));
            }

            pos_head = pos_tail + 1;
        }

        let command_and_params = &message[pos_head..];

        if let Some(i) = command_and_params.find(' ') {
            msg.command = Some(command_and_params[..i].to_owned());

            let params_string: &str = &command_and_params[i + 1..];
            let text_loc = params_string.find(':');
            let mut params: Vec<String> = Vec::new();

            match text_loc {
                Some(0) => {
                    params.push(params_string[1..].to_owned());
                }
                Some(loc) => {
                    params = params_string[..loc - 1]
                        .split_ascii_whitespace()
                        .map(|s| s.to_string())
                        .collect();
                    params.push(params_string[loc + 1..].to_owned());
                }
                None => {
                    params = params_string
                        .split_ascii_whitespace()
                        .map(|s| s.to_string())
                        .collect();
                }
            }

            msg.params = Some(params);
        } else {
            msg.command = Some(command_and_params.to_owned());
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
            Some(Prefix::Nick(
                "715209".to_string(),
                "715209".to_string(),
                "715209.tmi.twitch.tv".to_string()
            ))
        );
        assert_eq!(parsed.command, Some("PRIVMSG".to_string()));
        assert_eq!(
            parsed.params,
            Some(vec!["#715209".to_string(), "hello".to_string()])
        );
    }

    #[test]
    fn normal_message_no_tags() {
        let parsed =
            Message::parse(":715209!715209@715209.tmi.twitch.tv PRIVMSG #715209 :hello").unwrap();

        assert_eq!(parsed.tags, None);
        assert_eq!(
            parsed.prefix,
            Some(Prefix::Nick(
                "715209".to_string(),
                "715209".to_string(),
                "715209.tmi.twitch.tv".to_string()
            ))
        );
        assert_eq!(parsed.command, Some("PRIVMSG".to_string()));
        assert_eq!(
            parsed.params,
            Some(vec!["#715209".to_string(), "hello".to_string()])
        );
    }

    #[test]
    fn ping() {
        let parsed = Message::parse("PING :tmi.twitch.tv").unwrap();

        assert_eq!(parsed.tags, None);
        assert_eq!(parsed.prefix, None);
        assert_eq!(parsed.command, Some("PING".to_string()));
        assert_eq!(parsed.params, Some(vec!["tmi.twitch.tv".to_string()]));
    }

    #[test]
    fn no_params() {
        let parsed = Message::parse("@badge-info=;badges=;color=#008000;display-name=715209;emote-sets=0,33563,231890,300206296,300242181;user-id=21621987;user-type= :tmi.twitch.tv GLOBALUSERSTATE").unwrap();

        assert_ne!(parsed.tags, None);
        assert_eq!(
            parsed.prefix,
            Some(Prefix::Servername("tmi.twitch.tv".to_string()))
        );
        assert_eq!(parsed.command, Some("GLOBALUSERSTATE".to_string()));
        assert_eq!(parsed.params, None);
    }
    #[test]
    fn tags_no_prefix() {
        let parsed = Message::parse("@badge-info=;badges=;color=#008000;display-name=715209;emote-sets=0,33563,231890,300206296,300242181;user-id=21621987;user-type= GLOBALUSERSTATE").unwrap();

        assert_ne!(parsed.tags, None);
        assert_eq!(parsed.prefix, None);
        assert_eq!(parsed.command, Some("GLOBALUSERSTATE".to_string()));
        assert_eq!(parsed.params, None);
    }
    #[test]
    fn tags_and_params_no_prefix() {
        let parsed = Message::parse("@badge-info=;badges=;color=#008000;display-name=715209;emote-sets=0,33563,231890,300206296,300242181;user-id=21621987;user-type= PRIVMSG #715209 :hello").unwrap();

        assert_ne!(parsed.tags, None);
        assert_eq!(parsed.prefix, None);
        assert_eq!(parsed.command, Some("PRIVMSG".to_string()));
        assert_eq!(
            parsed.params,
            Some(vec!["#715209".to_string(), "hello".to_string()])
        );
    }
    #[test]
    fn only_command() {
        let parsed = Message::parse("PRIVMSG").unwrap();

        assert_eq!(parsed.tags, None);
        assert_eq!(parsed.prefix, None);
        assert_eq!(parsed.command, Some("PRIVMSG".to_string()));
        assert_eq!(parsed.params, None);
    }

    #[test]
    fn nothing_to_parse() {
        let parsed = Message::parse("");

        assert!(parsed.is_err(), "Nothing found to parse");
    }

    #[test]
    fn only_tags() {
        let parsed = Message::parse("@badge-info=;badges=;color=#008000;display-name=715209;emote-sets=0,33563,231890,300206296,300242181;user-id=21621987;user-type=");

        assert!(parsed.is_err(), "No command found");
    }
}
