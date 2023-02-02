///   A 16-bit 'flags' field split into (high to low bits)
/// 
///     1-bit assume-valid flag
///     1-bit extended flag (must be zero in version 2)
///     2-bit stage (during merge)
///     12-bit name length if the length is less than 0xFFF; otherwise 0xFFF
///     is stored in this field.
///     (Version 3 or later) A 16-bit field, only applicable if the
///     "extended flag" above is 1, split into (high to low bits).
///     1-bit reserved for future
///     1-bit skip-worktree flag (used by sparse checkout)
///     1-bit intent-to-add flag (used by "git add -N")
///     13-bit unused, must be zero
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub struct EntryFlags {
    pub (super) basic_flags: u16,
    pub (super) ext_flags: Option<u16>,
}

pub (super) const MASK_ASSUME_VALID: u16      = 0b1000_0000_0000_0000;
pub (super) const MASK_EXTENDED: u16          = 0b0100_0000_0000_0000;
pub (super) const MASK_STAGE: u16             = 0b0011_0000_0000_0000;
pub (super) const MASK_NAME_LEN: u16          = 0b0000_1111_1111_1111;
// pub (super) const MASK_EXT_RESERVED: u16   = 0b1000_0000_0000_0000;
pub (super) const MASK_EXT_SKIP_WORKTREE: u16 = 0b0100_0000_0000_0000;
pub (super) const MASK_EXT_INTENT_TO_ADD: u16 = 0b0010_0000_0000_0000;
// pub (super) const MASK_EXT_UNUSED: u16     = 0b0001_1111_1111_1111;

impl EntryFlags {
    pub fn new(name: &str) -> EntryFlags {
        let mut flags = EntryFlags { basic_flags: 0, ext_flags: None };

        let name_len = std::cmp::max(name.len(), 0xFFF);
        flags.set_name_len(name_len.try_into().unwrap());

        flags
    }

    pub fn get_assume_valid(&self) -> bool {
        return (self.basic_flags & MASK_ASSUME_VALID) != 0;
    }

    pub fn set_assume_valid(&mut self) {
        self.basic_flags |= MASK_ASSUME_VALID;
    }

    pub fn clear_assume_valid(&mut self) {
        self.basic_flags &= !MASK_ASSUME_VALID;
    }

    pub fn get_extended(&self) -> bool {
        return (self.basic_flags & MASK_EXTENDED) != 0;
    }

    pub fn set_extended(&mut self) {
        self.basic_flags |= MASK_EXTENDED;
        self.ext_flags = Some(0);
    }

    pub fn clear_extended(&mut self) {
        self.basic_flags &= !MASK_EXTENDED;
        self.ext_flags = None;
    }

    pub fn get_stage(&self) -> () {
        match self.basic_flags & MASK_STAGE {
            0b0000_0000_0000_0000 => (),
            0b0001_0000_0000_0000 => (),
            0b0010_0000_0000_0000 => (),
            0b0011_0000_0000_0000 => (),
            _ => (),
        }
    }

    pub fn set_stage(&mut self, _stage: ()) {
        panic!("not implemented");
    }

    pub fn get_name_len(&self) -> u16 {
        return self.basic_flags & MASK_NAME_LEN;
    }

    pub fn set_name_len(&mut self, value: u16) {
        if value > 0x0FFF {
            panic!("Name len cannot be more than 12 bits");
        }

        self.basic_flags &= !MASK_NAME_LEN;
        self.basic_flags |= value;
    }

    pub fn get_skip_worktree(&self) -> bool {
        return (self.ext_flags.unwrap() & MASK_EXT_SKIP_WORKTREE) != 0;
    }

    pub fn set_skip_worktree(&mut self) {
        *self.ext_flags.as_mut().unwrap() |= MASK_EXT_SKIP_WORKTREE;
    }

    pub fn clear_skip_worktree(&mut self) {
        *self.ext_flags.as_mut().unwrap() &= !MASK_EXT_SKIP_WORKTREE;
    }

    pub fn get_intent_to_add(&self) -> bool {
        return (self.ext_flags.unwrap() & MASK_EXT_INTENT_TO_ADD) != 0;
    }

    pub fn set_intent_to_add(&mut self) {
        *self.ext_flags.as_mut().unwrap() |= MASK_EXT_INTENT_TO_ADD;
    }

    pub fn clear_intent_to_add(&mut self) {
        *self.ext_flags.as_mut().unwrap() &= !MASK_EXT_INTENT_TO_ADD;
    }
}
