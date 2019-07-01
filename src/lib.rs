use std::collections::HashMap;

/*
<message>  ::= [':' <prefix> <SPACE> ] <command> <params> <crlf>
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
    pub fn parse(message: String) -> Result<Message, &'static str> {
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

        if let None = split_message.get(_loc) {
            return Err("No command found");
        }

        // command
        msg.command = Some(split_message[_loc].to_owned());
        _loc += 1;

        // check if there are any params
        if _loc == split_message.len() {
            return Ok(msg);
        }

        let mut params = Vec::new();

        while _loc < split_message.len() && split_message[_loc][..1] != ":".to_string() {
            params.push(split_message[_loc].to_owned());
            _loc += 1;
        }

        if _loc < split_message.len() && split_message[_loc][..1] == ":".to_string() {
            params.push(split_message[_loc..split_message.len()].join(" ")[1..].to_string());
        }

        msg.params = Some(params);

        Ok(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::Message;

    #[test]
    fn normal_message() {
        let parsed = Message::parse(String::from("@badge-info=;badges=broadcaster/1;color=#008000;display-name=715209;emotes=;flags=;id=8a90aa05-eea3-4699-84eb-1d4c65b85f94;mod=0;room-id=21621987;subscriber=0;tmi-sent-ts=1559891010190;turbo=0;user-id=21621987;user-type= :715209!715209@715209.tmi.twitch.tv PRIVMSG #715209 :hello")).unwrap();

        assert_ne!(parsed.tags, None);
        assert_eq!(
            parsed.prefix,
            Some(String::from(":715209!715209@715209.tmi.twitch.tv"))
        );
        assert_eq!(parsed.command, Some(String::from("PRIVMSG")));
        assert_eq!(
            parsed.params,
            Some(vec![String::from("#715209"), String::from("hello")])
        );
    }

    #[test]
    fn normal_message_no_tags() {
        let parsed = Message::parse(String::from(
            ":715209!715209@715209.tmi.twitch.tv PRIVMSG #715209 :hello",
        ))
        .unwrap();

        assert_eq!(parsed.tags, None);
        assert_eq!(
            parsed.prefix,
            Some(String::from(":715209!715209@715209.tmi.twitch.tv"))
        );
        assert_eq!(parsed.command, Some(String::from("PRIVMSG")));
        assert_eq!(
            parsed.params,
            Some(vec![String::from("#715209"), String::from("hello")])
        );
    }

    #[test]
    fn ping() {
        let parsed = Message::parse(String::from("PING :tmi.twitch.tv")).unwrap();

        assert_eq!(parsed.tags, None);
        assert_eq!(parsed.prefix, None);
        assert_eq!(parsed.command, Some(String::from("PING")));
        assert_eq!(parsed.params, Some(vec![String::from("tmi.twitch.tv")]));
    }

    #[test]
    fn no_params() {
        let parsed = Message::parse(String::from("@badge-info=;badges=;color=#008000;display-name=715209;emote-sets=0,33563,231890,300206296,300242181;user-id=21621987;user-type= :tmi.twitch.tv GLOBALUSERSTATE")).unwrap();

        assert_ne!(parsed.tags, None);
        assert_eq!(parsed.prefix, Some(String::from(":tmi.twitch.tv")));
        assert_eq!(parsed.command, Some(String::from("GLOBALUSERSTATE")));
        assert_eq!(parsed.params, None);
    }

    #[test]
    fn no_command() {
        let parsed = Message::parse(String::from("@badge-info=;badges=;color=#008000;display-name=715209;emote-sets=0,33563,231890,300206296,300242181;user-id=21621987;user-type= :tmi.twitch.tv"));

        assert!(parsed.is_err(), "No command found");
    }
}
