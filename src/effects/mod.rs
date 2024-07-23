use filter::Filter;

pub mod filter;

#[derive(Debug, Clone, Copy)]
pub enum Effects {
    Unused,
    Fliter(Filter),
}
