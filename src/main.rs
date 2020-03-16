use irc_parser::Message;

fn main() {
    let msg = Message::parse("@badge-info=;badges=broadcaster/1;color=#008000;display-name=715209;emotes=;flags=;id=6589be01-3810-4728-99d9-f26bdc43c81d;mod=0;room-id=21621987;subscriber=0;tmi-sent-ts=1559707099327;turbo=0;user-id=21621987;user-type= :715209!715209@715209.tmi.twitch.tv PRIVMSG #715209 :Hello");
    println!("{:#?}", msg)
}
