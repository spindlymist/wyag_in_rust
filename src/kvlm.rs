use ordered_multimap::ListOrderedMultimap;
use itertools::Itertools;
use thiserror::Error;

use crate::Result;

/// Parses a "key value list with message" into a map. The message is stored under the blank key `""`.
/// 
/// # Format
/// 
/// `//` indicates a note and is not part of the format. The trailing whitespace below is not significant,
/// but the parser will include trailing whitespace in the value except for the final newline.
/// 
/// ```text
/// key_1 value_1 line 1     // a space separates the key from the value
///  value_1 line 2          // a leading space continues the value from the previous line
///  ...                  
///  value_1 line N_1     
/// ...                   
/// key_M value_M line 1         
///  value_M line 2       
///  ...                  
///  value_M line N_M     
///                          // a blank line marks the end of the header
/// message line 1        
/// ...                   
/// message line N        
/// ```
/// 
/// # Examples
/// 
/// ```
/// use wyag::kvlm;
/// 
/// let input = "\
/// name Robin
/// address 1234 Rager Ln.
///  Corvallis, OR 97333
/// languages English
/// languages Rust
///
/// This is my message.
/// It has two lines.";
/// 
/// let map = kvlm::parse(input).expect("input should be valid");
/// 
/// assert_eq!( vec!["name", "address", "languages", ""], map.keys().collect::<Vec<&String>>() );
/// assert_eq!( "Robin", map.get("name").unwrap() );
/// assert_eq!( "1234 Rager Ln.\nCorvallis, OR 97333", map.get("address").unwrap() );
/// assert_eq!( vec!["English", "Rust"], map.get_all("languages").collect::<Vec<&String>>() );
/// assert_eq!( "This is my message.\nIt has two lines.", map.get("").unwrap() );
/// ```
pub fn parse(data: &str) -> Result<ListOrderedMultimap<String, String>> {
    let mut map = ListOrderedMultimap::new();

    let (header, message) = match data.split_once("\n\n") {
        Some((header, message)) => (header, message),
        None => return Err(KvlmError::MissingMessage.into()),
    };
    let mut header_lines = header.lines();
    
    while let Some((key, value)) = next_entry(&mut header_lines)? {
        map.append(key, value);
    }

    map.insert(String::from(""), String::from(message));

    Ok(map)
}

fn next_entry<'a, I>(lines: &mut I) -> Result<Option<(String, String)>>
where
    I: Iterator<Item = &'a str> + Clone
{
    let first_line = match lines.next() {
        Some(val) => val,
        None => return Ok(None),
    };

    let (key, first_value_line) = match first_line.split_once(' ') {
        Some(val) => val,
        None => return Err(KvlmError::InvalidEntry.into()),
    };

    let mut value_lines = vec![first_value_line];
    value_lines.extend(
        lines.take_while_ref(|line| line.starts_with(' '))
            .map(|line| &line[1..])
    );

    Ok(Some((
        String::from(key),
        value_lines.join("\n"),
    )))
}

/// Serializes a ListOrderedMultimap into a "key value list with message."
/// 
/// The message should be stored under the blank key `""`. See [`kvlm_parse`] for detailed format specification.
/// 
/// # Examples
/// 
/// ```
/// use wyag::kvlm;
/// 
/// let mut map = ordered_multimap::ListOrderedMultimap::<String, String>::new();
/// map.insert("name".to_owned(), "Robin".to_owned());
/// map.insert("address".to_owned(), "1234 Rager Ln.\nCorvallis, OR 97333".to_owned());
/// map.append("languages".to_owned(), "English".to_owned());
/// map.append("languages".to_owned(), "Rust".to_owned());
/// map.insert("".to_owned(), "This is my message.\nIt has two lines.".to_owned());
/// 
/// let serialized = kvlm::serialize(&map);
/// 
/// assert_eq!(serialized, "\
/// name Robin
/// address 1234 Rager Ln.
///  Corvallis, OR 97333
/// languages English
/// languages Rust
///
/// This is my message.
/// It has two lines.");
/// ```
pub fn serialize(kvlm: &ListOrderedMultimap<String, String>) -> String {
    let header = kvlm.pairs()
        .filter(|(key, _)| !key.is_empty())
        .map(|(key, values)| {
            values.map(|value| {
                let value = value.replace('\n', "\n ");
                format!("{key} {value}")
            }).join("\n")
        })
        .join("\n");

    let message = match kvlm.get("") {
        Some(message) => &message[..],
        None => "",
    };

    format!("{header}\n\n{message}")
}

#[derive(Error, Debug)]
pub enum KvlmError {
    #[error("The kvlm has no message (no blank line after list)")]
    MissingMessage,
    #[error("The kvlm has an invalid entry (no space after key)")]
    InvalidEntry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_values() {
        let map = parse("\
key1 this is value 1
key2 value 2
 has exactly
 three lines
key3 this is value 3

this is the message
with two lines").unwrap();
        {
            let keys: Vec<_> = map.keys().collect();
            assert_eq!(keys, vec!["key1", "key2", "key3", ""]);
        }
        {
            let key1_values: Vec<_> = map.get_all("key1").collect();
            assert_eq!(key1_values, vec!["this is value 1"]);
        }
        {
            let key2_values: Vec<_> = map.get_all("key2").collect();
            assert_eq!(key2_values, vec!["value 2\nhas exactly\nthree lines"]);
        }
        {
            let key3_values: Vec<_> = map.get_all("key3").collect();
            assert_eq!(key3_values, vec!["this is value 3"]);
        }
        {
            let msg_values: Vec<_> = map.get_all("").collect();
            assert_eq!(msg_values, vec!["this is the message\nwith two lines"]);
        }
    }

    #[test]
    fn parse_multi_values() {
        let map = parse("\
key1 this is value 1.1
 which has two lines
key2 this is value 2.1
key2 this is value 2.2
 which also has two lines
key1 this is value 1.2

this is the message").unwrap();
        {
            let keys: Vec<_> = map.keys().collect();
            assert_eq!(keys, vec!["key1", "key2", ""]);
        }
        {
            let key1_values: Vec<_> = map.get_all("key1").collect();
            assert_eq!(key1_values, vec!["this is value 1.1\nwhich has two lines", "this is value 1.2"]);
        }
        {
            let key2_values: Vec<_> = map.get_all("key2").collect();
            assert_eq!(key2_values, vec!["this is value 2.1", "this is value 2.2\nwhich also has two lines"]);
        }
        {
            let msg_values: Vec<_> = map.get_all("").collect();
            assert_eq!(msg_values, vec!["this is the message"]);
        }
    }

    #[test]
    fn parse_rejects_missing_key() {
        let map = parse("\
key1 is okay
key2hasnospace

this is the message");
        assert!(map.is_err());
    }

    #[test]
    fn parse_rejects_missing_message() {
        let map = parse("\
key1 is okay
key2 is too");
        assert!(map.is_err());
    }

    #[test]
    fn serialize_single_values() {
        let mut map = ListOrderedMultimap::new();
        map.insert("key1".to_owned(), "this is value 1".to_owned());
        map.insert("key2".to_owned(), "value 2\nhas exactly\nthree lines".to_owned());
        map.insert("key3".to_owned(), "this is value 3".to_owned());
        map.insert("".to_owned(), "this is the message\nwith two lines".to_owned());

        let serialized = serialize(&map);

        assert_eq!(serialized, "\
key1 this is value 1
key2 value 2
 has exactly
 three lines
key3 this is value 3

this is the message
with two lines");
    }

    #[test]
    fn serialize_multi_values() {
        let mut map = ListOrderedMultimap::new();
        map.insert("key1".to_owned(), "this is value 1.1\nwhich has two lines".to_owned());
        map.append("key1".to_owned(), "this is value 1.2".to_owned());
        map.insert("key2".to_owned(), "this is value 2.1".to_owned());
        map.append("key2".to_owned(), "this is value 2.2\nwhich also has two lines".to_owned());
        map.insert("".to_owned(), "this is the message".to_owned());

        let serialized = serialize(&map);

        assert_eq!(serialized, "\
key1 this is value 1.1
 which has two lines
key1 this is value 1.2
key2 this is value 2.1
key2 this is value 2.2
 which also has two lines

this is the message");
    }
}
