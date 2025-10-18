pub struct VacantEntry<'a, T> {
    option: &'a mut Option<T>,
}

impl<'a, T> VacantEntry<'a, T> {
    fn new(option: &'a mut Option<T>) -> Self {
        assert!(option.is_none());
        Self { option }
    }

    pub fn insert(self, value: T) -> &'a mut T {
        self.option.insert(value)
    }

    pub fn insert_entry(self, value: T) -> OccupiedEntry<'a, T> {
        let Self { option } = self;
        *option = Some(value);
        OccupiedEntry::new(option)
    }
}

pub struct OccupiedEntry<'a, T> {
    option: &'a mut Option<T>,
}

impl<'a, T> OccupiedEntry<'a, T> {
    fn new(option: &'a mut Option<T>) -> Self {
        assert!(option.is_some());
        Self { option }
    }

    pub fn get(&self) -> &T {
        self.option.as_ref().expect("OccupiedEntry is None?")
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.option.as_mut().expect("OccupiedEntry is None?")
    }

    pub fn insert(&mut self, value: T) -> T {
        self.option.replace(value).expect("OccupiedEntry is None?")
    }

    pub fn into_mut(self) -> &'a mut T {
        self.option.as_mut().expect("OccupiedEntry is None?")
    }

    pub fn remove(self) -> T {
        self.option.take().expect("OccupiedEntry is None?")
    }
}

pub enum Entry<'a, T> {
    Vacant(VacantEntry<'a, T>),
    Occupied(OccupiedEntry<'a, T>),
}

impl<'a, T> Entry<'a, T> {
    fn into_option_mut(self) -> &'a mut Option<T> {
        match self {
            Entry::Vacant(VacantEntry { option }) => option,
            Entry::Occupied(OccupiedEntry { option }) => option,
        }
    }

    pub fn and_modify(self, f: impl FnOnce(&mut T)) -> Self {
        let option = self.into_option_mut();
        if let Some(value) = option {
            f(value);
        }
        option.entry()
    }

    pub fn insert_entry(self, value: T) -> OccupiedEntry<'a, T> {
        match self {
            Entry::Occupied(mut entry) => {
                entry.insert(value);
                entry
            }
            Entry::Vacant(entry) => entry.insert_entry(value),
        }
    }

    pub fn or_default(self) -> &'a mut T
    where
        T: Default,
    {
        self.into_option_mut().get_or_insert_default()
    }

    pub fn or_insert(self, default: T) -> &'a mut T {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default),
        }
    }

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

pub trait OptionEntry: private::Sealed {
    type T;
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
