//! [`Entry`] for [`Option`]s.
//!
//! See [`OccupiedEntry::remove`] for why.
//!
//! Docs and interface are based on `btree_map::Entry`.

#![no_std]

/// [`None`]
pub struct VacantEntry<'a, T> {
    option: &'a mut Option<T>,
}

impl<'a, T> VacantEntry<'a, T> {
    fn new(option: &'a mut Option<T>) -> Self {
        assert!(option.is_none());
        Self { option }
    }

    /// Sets the value of the [`Option`], and returns a mutable reference to it.
    pub fn insert(self, value: T) -> &'a mut T {
        self.option.insert(value)
    }

    /// Sets the value of the [`Option`], and returns an [`OccupiedEntry`].
    pub fn insert_entry(self, value: T) -> OccupiedEntry<'a, T> {
        let Self { option } = self;
        *option = Some(value);
        OccupiedEntry::new(option)
    }
}

/// [`Some`]
pub struct OccupiedEntry<'a, T> {
    option: &'a mut Option<T>,
}

impl<'a, T> OccupiedEntry<'a, T> {
    fn new(option: &'a mut Option<T>) -> Self {
        assert!(option.is_some());
        Self { option }
    }

    /// Gets a reference to the value within [`Some`].
    pub fn get(&self) -> &T {
        self.option.as_ref().expect("OccupiedEntry is None?")
    }

    /// Gets a mutable reference to the value within [`Some`].
    ///
    /// If you need a reference to the [`OccupiedEntry`] that may outlive the destruction of the [`Entry`] value, see [`into_mut`].
    ///
    /// [`into_mut`]: Self::into_mut
    pub fn get_mut(&mut self) -> &mut T {
        self.option.as_mut().expect("OccupiedEntry is None?")
    }

    /// [`Option::replace`]. Returns `T` instead of `Option<T>`, like [`mem::replace(x, value)`]
    /// within `if let Some(x) = o` context.
    ///
    /// [`mem::replace(x, value)`]: core::mem::replace
    pub fn insert(&mut self, value: T) -> T {
        self.option.replace(value).expect("OccupiedEntry is None?")
    }

    /// Converts the entry into a mutable reference to its value.
    ///
    /// If you need multiple references to the [`OccupiedEntry`], see [`get_mut`].
    ///
    /// [`get_mut`]: Self::get_mut
    pub fn into_mut(self) -> &'a mut T {
        self.option.as_mut().expect("OccupiedEntry is None?")
    }

    /// [`Option::take`]. Returns `T` instead of `Option<T>`.
    ///
    /// This method is the main reason for this crate to exist: this allows avoiding doing
    /// double-checks in code that needs to do deferred [`take`].
    ///
    /// [`take`]: Option::take
    pub fn remove(self) -> T {
        self.option.take().expect("OccupiedEntry is None?")
    }
}

/// `&mut Option<T>` with strongly typed context of whether it's [`None`] or [`Some`] which provides
/// methods like [`OccupiedEntry::remove`].
pub enum Entry<'a, T> {
    /// [`None`]
    Vacant(VacantEntry<'a, T>),
    /// [`Some`]
    Occupied(OccupiedEntry<'a, T>),
}

impl<'a, T> Entry<'a, T> {
    fn into_option_mut(self) -> &'a mut Option<T> {
        match self {
            Entry::Vacant(VacantEntry { option }) => option,
            Entry::Occupied(OccupiedEntry { option }) => option,
        }
    }

    /// Provides in-place mutable access to a [`Some`] before any potential inserts into the option.
    pub fn and_modify(self, f: impl FnOnce(&mut T)) -> Self {
        let option = self.into_option_mut();
        if let Some(value) = option {
            f(value);
        }
        option.entry()
    }

    /// Sets the option to `Some(value)`, and returns an [`OccupiedEntry`].
    pub fn insert_entry(self, value: T) -> OccupiedEntry<'a, T> {
        match self {
            Entry::Occupied(mut entry) => {
                entry.insert(value);
                entry
            }
            Entry::Vacant(entry) => entry.insert_entry(value),
        }
    }

    /// Ensures a value is in the option by inserting the default value if [`None`], and returns a
    /// mutable reference to that (already present or default) value.
    pub fn or_default(self) -> &'a mut T
    where
        T: Default,
    {
        self.into_option_mut().get_or_insert_default()
    }

    /// Ensures a value is in the option by inserting `default` if [`None`], and returns a mutable
    /// reference to that (already present or `default`) value.
    pub fn or_insert(self, default: T) -> &'a mut T {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default),
        }
    }

    /// Ensures a value is in the option by inserting `default()` if [`None`], and returns a mutable
    /// reference to that (already present or `default()`) value.
    pub fn or_insert_with(self, default: impl FnOnce() -> T) -> &'a mut T {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default()),
        }
    }
}

mod private {
    pub trait Sealed {}
}

/// Extension trait for viewing [`Option`] as [`Entry`].
pub trait OptionEntry: private::Sealed {
    /// `T` in `Option<T>`.
    type T;
    /// View the current [`Option`] as an [`Entry`], primarily for [`OccupiedEntry::remove`].
    fn entry(&mut self) -> Entry<'_, Self::T>;
}

impl<T> private::Sealed for Option<T> {}

impl<T> OptionEntry for Option<T> {
    type T = T;

    fn entry(&mut self) -> Entry<'_, Self::T> {
        if self.is_none() {
            Entry::Vacant(VacantEntry::new(self))
        } else {
            Entry::Occupied(OccupiedEntry::new(self))
        }
    }
}
