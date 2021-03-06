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
    use crate::mutex::{Mutex};
    use std::thread;
    use core::ops::DerefMut;
    
    struct Node {
        x: u32,
    }
    impl Node {
        fn inc(&mut self) {
            self.x += 1;
        }
        fn get_data(&self) -> u32 {
            self.x
        }
    }

    #[test]
    fn test_mutex1() {
        static M: Mutex<Node> = Mutex::new(Node{x: 0});
        const J: u32 = 10;
        const K: u32 = 1;
        fn inc() {
            for _ in 0..J {
                {
                    let mut tmp = M.lock();
                    tmp.inc();
                }
            }
        }

        for _ in 0..K {
            thread::spawn(move|| { inc();}).join().unwrap();
            thread::spawn(move|| { inc();}).join().unwrap();
        }
        assert_eq!(M.lock().get_data(), 20);
    }

    #[test]
    fn test_mutex2() {

        static MUTEX: Mutex<u32> = Mutex::new(0);
        const N: u32 = 10;
        const T: u32 = 5;

        fn inc(i: u32) {
            for _ in 0..N {
                {
                    let mut xx = MUTEX.lock();
                    (*xx.deref_mut()) += i;
                }
            }
        }

        for _ in 0..T {
            thread::spawn(move|| { inc(1);}).join().unwrap();
            thread::spawn(move|| { inc(2);}).join().unwrap();
        }
        assert_eq!(*MUTEX.lock().deref_mut(), 150);
    }
}