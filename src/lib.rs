pub mod mutex;

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
    fn mutex_test() {
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
            thread::spawn(move|| { inc();});
            thread::spawn(move|| { inc();});
        }
    }
}
