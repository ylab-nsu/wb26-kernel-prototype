#![allow(unused)]

pub type Mutex<T> = spin::mutex::Mutex<T, spin::relax::Spin>;
pub type RwLock<T> = spin::rwlock::RwLock<T, spin::relax::Spin>;
pub type Once<T> = spin::once::Once<T, spin::relax::Spin>;
pub type LazyLock<T> = spin::lazylock::LazyLock<T, fn() -> T, spin::relax::Spin>;
pub type Barrier = spin::barrier::Barrier<spin::relax::Spin>;
