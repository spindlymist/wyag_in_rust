use ordered_multimap::ListOrderedMultimap;
use itertools::Itertools;

use crate::error::Error;

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
/// use wyag::kvlm::kvlm_parse;
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
/// let map = kvlm_parse(input).expect("input should be valid");
/// 
/// assert_eq!( vec!["name", "address", "languages", ""], map.keys().collect::<Vec<&String>>() );
/// assert_eq!( "Robin", map.get("name").unwrap() );
/// assert_eq!( "1234 Rager Ln.\nCorvallis, OR 97333", map.get("address").unwrap() );
/// assert_eq!( vec!["English", "Rust"], map.get_all("languages").collect::<Vec<&String>>() );
/// assert_eq!( "This is my message.\nIt has two lines.", map.get("").unwrap() );
/// ```
pub fn parse(data: &str) -> Result<ListOrderedMultimap<String, String>, Error> {
    let mut map = ListOrderedMultimap::new();

    let (header, message) = match data.split_once("\n\n") {
        Some((header, message)) => (header, message),
        None => return Err(Error::BadKVLMFormat),
    };
    let mut header_lines = header.lines();
    
    while let Some((key, value)) = next_entry(&mut header_lines) {
        map.append(key, value);
    }

    map.insert(String::from(""), String::from(message));

    Ok(map)
}

fn next_entry<'a, I>(lines: &mut I) -> Option<(String, String)>
where
    I: Iterator<Item = &'a str> + Clone
{
    let first_line = lines.next()?;
    let (key, first_value_line) = first_line.split_once(' ')?;

    let mut value_lines = vec![first_value_line];
    value_lines.extend(
        lines.take_while_ref(|line| line.starts_with(' '))
        .map(|line| &line[1..])
    );

    Some((
        String::from(key),
        value_lines.join("\n"),
    ))
}

/// Serializes a ListOrderedMultimap into a "key value list with message."
/// 
/// The message should be stored under the blank key `""`. See [`kvlm_parse`] for detailed format specification.
/// 
/// # Examples
/// 
/// ```
/// use wyag::kvlm::kvlm_serialize;
/// 
/// let mut map = ordered_multimap::ListOrderedMultimap::<String, String>::new();
/// map.insert("name".to_owned(), "Robin".to_owned());
/// map.insert("address".to_owned(), "1234 Rager Ln.\nCorvallis, OR 97333".to_owned());
/// map.append("languages".to_owned(), "English".to_owned());
/// map.append("languages".to_owned(), "Rust".to_owned());
/// map.insert("".to_owned(), "This is my message.\nIt has two lines.".to_owned());
/// 
/// let serialized = kvlm_serialize(&map);
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
