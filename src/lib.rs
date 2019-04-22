pub mod mutex;
pub mod rwlock;

#[cfg(test)]
mod tests {
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
        use crate::mutex::Mutex;
        use std::thread;

        static M: Mutex<Node> = Mutex::new(Node{x: 0});
        const J: u32 = 10;
        const K: u32 = 1;
        fn inc() {
            for _ in 0..J {
                {
                    let mut tmp = M.lock();
                    tmp.inc();
                    println!("x: {}", tmp.get_data());
                }
            }
        }

        for _ in 0..K {
            thread::spawn(move|| { inc();}).join().unwrap();
            thread::spawn(move|| { inc();}).join().unwrap();
        }
    }

    #[test]
    fn test_mutex2() {
        use crate::mutex::{Mutex};
        use std::thread;
        use core::ops::DerefMut;

        static MUTEX: Mutex<u32> = Mutex::new(0);
        const N: u32 = 10;
        const T: u32 = 5;

        fn inc(i: u32) {
            for _ in 0..N {
                {
                    let mut xx = MUTEX.lock();
                    (*xx.deref_mut()) += i;
                    println!("tmp: {}", xx.deref_mut());
                }
            }
        }

        for _ in 0..T {
            thread::spawn(move|| { inc(1);}).join().unwrap();
            thread::spawn(move|| { inc(2);}).join().unwrap();
        }

    }
}
