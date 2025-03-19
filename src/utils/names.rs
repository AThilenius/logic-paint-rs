/// Takes in a name and makes it unique against the `existing` names by either incrementing an
/// existing number at the end of the name (if provided) or appending 1 to the end and incrementing
/// that until the name is unique.
pub fn make_name_unique(name: String, existing: Vec<String>) -> String {
    if !existing.contains(&name) {
        return name;
    }

    let (name, range_start) = if let Some(index) = name.rfind(|c: char| c.is_ascii_digit()) {
        // Text ends in digits, so increment those.
        (
            name[0..index].to_string(),
            name[index..].parse::<usize>().unwrap(),
        )
    } else {
        // Otherwise start the range at 1
        (name, 1_usize)
    };

    let mut unique_name = name.clone();

    for i in range_start.. {
        unique_name = format!("{}{}", name, i);

        if !existing.contains(&unique_name) {
            // Found a unique name
            break;
        }
    }
    unique_name
}
