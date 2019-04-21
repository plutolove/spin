use core::sync::atomic::{AtomicBool, Ordering, spin_loop_hint};
use core::cell::UnsafeCell;
use core::ops::{Drop, Deref, DerefMut};

pub struct Mutex<T: ?Sized> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    pub const fn new(udata: T) -> Mutex<T> {
        Self {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(udata)
        }
    }

    pub fn into_inner(self) -> T {
        // We know statically that there are no outstanding references to
        // `self` so there's no need to lock.
        let Mutex { data, .. } = self;
        data.into_inner()
    }

}

impl<T: ?Sized> Mutex<T> {

    fn get_lock(&self) {
        //lock初始化为false，加锁时设置为true，compare_and_swap函数返回false，结束while循环，加锁成功
        //当lock为true时，则说明有另一个线程持有锁，进入第一个while循环，第二个while循环load结果为true，执行spin_loop_hint，线程忙等
        //当线程释放锁时，lock设置成false，则第二个while循环load为false，跳到第一个while循环，加锁成功
        while self.lock.compare_and_swap(false, true, Ordering::Acquire) != false {
            while self.lock.load(Ordering::Relaxed) {
                spin_loop_hint();
            }
        }
    }

    pub fn lock(&self) -> Lock<T> {
        self.get_lock();
        Lock {
            lock: &self.lock,
            data: unsafe{
                &mut *self.data.get()
            },
        }
    }

    pub unsafe fn force_unlock(&self) {
        self.lock.store(false, Ordering::Release);
    }

    pub fn try_lock(&self) -> Option<Lock<T>> {
        if !self.lock.compare_and_swap(false, true, Ordering::Acquire) {
            Some(
                Lock {
                    lock: &self.lock,
                    data: unsafe {
                        &mut *self.data.get()
                    }
                }
            )
        } else {
            None
        }
    }
}

pub struct Lock<'a, T: ?Sized + 'a> {
    lock: &'a AtomicBool,
    data: &'a mut T,
}

unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}

impl <'a, T: ?Sized> Deref for Lock<'a, T> {
    type Target = T;
    fn deref(&self) -> &T { &*self.data }
}

impl <'a, T: ?Sized> DerefMut for Lock<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.data
    }
}

impl <'a, T: ?Sized> Drop for Lock<'a, T> {
    fn drop(&mut self) {
        self.lock.store(false, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn mutex_test() {
        use super::Mutex;
        use std::thread;
        use std::sync::mpsc::channel;

        static M: Mutex<()>  = Mutex::new(());
        static mut CNT: u32 = 0;
        const J: u32 = 1000;
        const K: u32 = 3;

        fn inc() {
            for _ in 0..J {
                unsafe {
                    let _g = M.lock();
                    CNT += 1;
                }
            }
        }

        let (tx, rx) = channel();
        for _ in 0..K {
            let tx2 = tx.clone();
            thread::spawn(move|| { inc(); tx2.send(()).unwrap(); });
            let tx2 = tx.clone();
            thread::spawn(move|| { inc(); tx2.send(()).unwrap(); });
        }
        drop(tx);
        for _ in 0..2 * K {
            rx.recv().unwrap();
        }
    }
}
