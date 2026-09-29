mod entry_manager;
mod interface;

pub(crate) use entry_manager::EntryError;
pub(crate) use entry_manager::EntryStatus;
pub(crate) use entry_manager::EntryTree;
pub(crate) use entry_manager::compare_dirs;
pub(crate) use entry_manager::compare_files;

pub use interface::compare;
pub use interface::compare_verbose;
pub use entry_manager::update;
