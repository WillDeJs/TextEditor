 #![allow(            
    clippy::missing_docs_in_private_items,            
    clippy::implicit_return,            
    clippy::shadow_reuse,            
    clippy::print_stdout,            
    clippy::wildcard_enum_match_arm,            
    clippy::else_if_without_else            
)]
pub mod editor;
pub mod terminal;
pub mod document;
pub mod row;
pub mod filetype;
pub mod highlighting;