use core::fmt;
use core::ops::Range;

use zerocopy::{AsBytes, BigEndian, FromBytes, U64};

pub type ZerocopyWord = U64<BigEndian>;

pub type NativeWord = u64;

const ARBITRARY_WORD: U64<BigEndian> = U64::ZERO;

const IS_PRESENT: u8 = 1;
const IS_NOT_PRESENT: u8 = 0;

#[derive(Debug, Clone)]
pub struct InvalidZerocopyOptionTag {
    pub tag: u8,
}

impl fmt::Display for InvalidZerocopyOptionTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid zerocopy tag: {}", self.tag)
    }
}

#[derive(Debug, Clone)]
pub enum InvalidZerocopyOptionTagOr<T> {
    InvalidZerocopyOptionTag(InvalidZerocopyOptionTag),
    Or(T),
}

impl<T: fmt::Display> fmt::Display for InvalidZerocopyOptionTagOr<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidZerocopyOptionTag(err) => err.fmt(f),
            Self::Or(err) => err.fmt(f),
        }
    }
}

//

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, AsBytes, FromBytes)]
pub struct ZerocopyWordRange {
    start: ZerocopyWord,
    end: ZerocopyWord,
}

impl ZerocopyWordRange {
    const ARBITRARY: Self = Self {
        start: U64::ZERO,
        end: U64::ZERO,
    };

    pub(crate) fn try_into_native<T: TryFrom<NativeWord>>(&self) -> Result<Range<T>, T::Error> {
        self.try_into()
    }

    #[cfg_attr(not(feature = "alloc"), allow(dead_code))]
    pub(crate) fn try_from_native<T: TryInto<NativeWord> + Copy>(
        native: &Range<T>,
    ) -> Result<Self, T::Error> {
        Self::try_from(native)
    }
}

impl<T: TryFrom<NativeWord>> TryFrom<&ZerocopyWordRange> for Range<T> {
    type Error = T::Error;

    fn try_from(range: &ZerocopyWordRange) -> Result<Self, Self::Error> {
        Ok(range.start.get().try_into()?..range.end.get().try_into()?)
    }
}

impl<T: TryInto<NativeWord> + Copy> TryFrom<&Range<T>> for ZerocopyWordRange {
    type Error = T::Error;

    fn try_from(range: &Range<T>) -> Result<Self, Self::Error> {
        Ok(ZerocopyWordRange {
            start: U64::new(range.start.try_into()?),
            end: U64::new(range.end.try_into()?),
        })
    }
}

//

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, AsBytes, FromBytes)]
pub struct ZerocopyOptionWord {
    is_present: u8,
    value: ZerocopyWord,
}

impl ZerocopyOptionWord {
    pub(crate) fn try_into_native<T: TryFrom<NativeWord>>(
        &self,
    ) -> Result<Option<T>, InvalidZerocopyOptionTagOr<T::Error>> {
        <Option<ZerocopyWord>>::try_from(self)
            .map_err(InvalidZerocopyOptionTagOr::InvalidZerocopyOptionTag)
            .map(|option_zerocopy_word| {
                option_zerocopy_word
                    .map(U64::get)
                    .map(TryInto::try_into)
                    .transpose()
                    .map_err(InvalidZerocopyOptionTagOr::Or)
            })
            .flatten()
    }

    #[cfg_attr(not(feature = "alloc"), allow(dead_code))]
    pub(crate) fn try_from_native<T: TryInto<NativeWord> + Copy>(
        native: &Option<T>,
    ) -> Result<Self, T::Error> {
        native
            .as_ref()
            .map(|try_into_native_word| (*try_into_native_word).try_into().map(U64::new))
            .transpose()
            .map(|x| x.as_ref().into())
    }
}

impl From<Option<&ZerocopyWord>> for ZerocopyOptionWord {
    fn from(option: Option<&ZerocopyWord>) -> Self {
        match option {
            Some(value) => Self {
                is_present: IS_PRESENT,
                value: *value,
            },
            None => Self {
                is_present: IS_NOT_PRESENT,
                value: ARBITRARY_WORD,
            },
        }
    }
}

impl TryFrom<&ZerocopyOptionWord> for Option<ZerocopyWord> {
    type Error = InvalidZerocopyOptionTag;

    fn try_from(zerocopy_option: &ZerocopyOptionWord) -> Result<Self, Self::Error> {
        Ok(match zerocopy_option.is_present {
            IS_PRESENT => Some(zerocopy_option.value),
            IS_NOT_PRESENT => None,
            tag => return Err(InvalidZerocopyOptionTag { tag }),
        })
    }
}

//

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, AsBytes, FromBytes)]
pub struct ZerocopyOptionWordRange {
    is_present: u8,
    value: ZerocopyWordRange,
}

impl ZerocopyOptionWordRange {
    pub(crate) fn try_into_native<T: TryFrom<NativeWord>>(
        &self,
    ) -> Result<Option<Range<T>>, InvalidZerocopyOptionTagOr<T::Error>> {
        <Option<ZerocopyWordRange>>::try_from(self)
            .map_err(InvalidZerocopyOptionTagOr::InvalidZerocopyOptionTag)
            .map(|option_zerocopy_word_range| {
                option_zerocopy_word_range
                    .map(|zerocopy_word_range| <Range<T>>::try_from(&zerocopy_word_range))
                    .transpose()
                    .map_err(InvalidZerocopyOptionTagOr::Or)
            })
            .flatten()
    }

    #[cfg_attr(not(feature = "alloc"), allow(dead_code))]
    pub(crate) fn try_from_native<T: TryInto<NativeWord> + Copy>(
        native: &Option<Range<T>>,
    ) -> Result<Self, T::Error> {
        native
            .as_ref()
            .map(|range_try_into_native_word| {
                ZerocopyWordRange::try_from_native(&range_try_into_native_word)
            })
            .transpose()
            .map(|x| x.as_ref().into())
    }
}

impl From<Option<&ZerocopyWordRange>> for ZerocopyOptionWordRange {
    fn from(option: Option<&ZerocopyWordRange>) -> Self {
        match option {
            Some(value) => Self {
                is_present: IS_PRESENT,
                value: value.clone(),
            },
            None => Self {
                is_present: IS_NOT_PRESENT,
                value: ZerocopyWordRange::ARBITRARY,
            },
        }
    }
}

impl TryFrom<&ZerocopyOptionWordRange> for Option<ZerocopyWordRange> {
    type Error = InvalidZerocopyOptionTag;

    fn try_from(zerocopy_option: &ZerocopyOptionWordRange) -> Result<Self, Self::Error> {
        Ok(match zerocopy_option.is_present {
            IS_PRESENT => Some(zerocopy_option.value.clone()),
            IS_NOT_PRESENT => None,
            tag => return Err(InvalidZerocopyOptionTag { tag }),
        })
    }
}
