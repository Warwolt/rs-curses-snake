// We want to just update TYPE and know what item is currently highlighted,
// and if any item was selected.
// User have 3 actions, `next_item`, `previous_item` and `select_current`

/// Used for a list of items that can be navigated and selected by the user
///
/// `index` is the position in the list, and `phantom` is used to store the
/// type used to represent item list choices, presumed to be C-style enum.
#[derive(Debug)]
pub struct ItemList<T: Copy> {
    index: usize,
    items: Vec<T>,
}

impl<T: Copy> ItemList<T> {
    pub fn new<I: Iterator<Item = T>>(item_iter: I, index: usize) -> Self {
        ItemList {
            index,
            items: item_iter.collect(),
        }
    }

    pub fn move_forward(&mut self) {
        self.index = usize::min(self.index + 1, self.items.len() - 1);
    }

    pub fn move_back(&mut self) {
        self.index = self.index.saturating_sub(1);
    }

    pub fn current_item(&self) -> T {
        self.items[self.index]
    }

    pub fn current_index(&self) -> usize {
        self.index
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use enum_iterator::IntoEnumIterator;

    #[derive(Debug, Copy, Clone, IntoEnumIterator)]
    enum TestEnum {
        A = 0xAA,
        B = 0xBB,
        C = 0xCC,
    }

    #[test]
    fn initial_selection_is_first_variant() {
        let list = ItemList::new(TestEnum::into_enum_iter(), 0);
        assert_eq!(list.current_item(), TestEnum::A);
    }

    #[test]
    fn next_grabs_item_in_number_order() {
        let mut list = ItemList::new(TestEnum::into_enum_iter(), 0);
        list.move_forward();
        assert_eq!(list.current_item(), TestEnum::B);
    }

    #[test]
    fn previous_from_initial_is_just_initial() {
        let mut list = ItemList::new(TestEnum::into_enum_iter(), 0);
        assert_eq!(list.current_item(), TestEnum::A);
        list.move_back();
        assert_eq!(list.current_item(), TestEnum::A);
    }

    #[test]
    fn next_from_final_is_just_final() {
        let mut list = ItemList::new(TestEnum::into_enum_iter(), 0);
        list.move_forward(); // B
        list.move_forward(); // C
        list.move_forward(); // C
        assert_eq!(list.current_item(), TestEnum::C);
    }
}
