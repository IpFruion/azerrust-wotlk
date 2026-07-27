use strum::FromRepr;

#[derive(Debug, FromRepr, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ComparisonType {
    Eq = 0,
    High = 1,
    Low = 2,
    HighEq = 3,
    LowEq = 4,
}

impl ComparisonType {
    pub fn compare<T: PartialOrd>(&self, val1: T, val2: T) -> bool {
        match self {
            ComparisonType::Eq => val1 == val2,
            ComparisonType::High => val1 > val2,
            ComparisonType::Low => val1 < val2,
            ComparisonType::HighEq => val1 >= val2,
            ComparisonType::LowEq => val1 <= val2,
        }
    }
}
