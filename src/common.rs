use crate::datatypes::PostCard;


pub fn items() -> Vec<PostCard> {
    vec![
    PostCard::new(
    "About Zeth".to_string(),
    "Zeth is the most important resource, arguably, in the world".to_string(),
    "19th August 2020".to_string(),
    "a".to_string(),
    1,
    ),
    PostCard::new(
    "About Poop".to_string(),
    "Poop is the most important resource, arguably, in the world".to_string(),
    "19th August 2020".to_string(),
    "a".to_string(),
    2,
    ),
    PostCard::new(
    "About Zeth's Poop".to_string(),
    "Zeth's Poop is the most important resource, in the whole universe".to_string(),
    "19th August 2020".to_string(),
    "a".to_string(),
    3,
    )
    ]
}