use std::{
    sync::{mpsc, Arc, Mutex}, 
    thread,
};

/// thread pool을 관리
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// ThreadPool instance를 생성하는 associated function
    /// 
    /// `size`는 pool 내의 worker 개수
    /// 
    /// ## Panic
    /// 
    /// size가 0일 경우 panic
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        // channel 생성
        let (sender, receiver) = mpsc::channel();

        // multiple producer, single consumer 원칙 때문에 receiver 복제는 불가능
        // channel queue로부터 job을 가져오는 작업은 receiver를 변경하는 작업임
        // Arc type은 여러 worker가 하나의 receiver를 소유할 수 있도록 함
        // Mutex type은 한번에 오직 한 worker만 receiver로부터 job을 얻을 수 있게 함 -> data race 방지
        let receiver = Arc::new(Mutex::new(receiver));

        // worker 생성
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            // 각 worker는 receiver를 holding
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        
        // thread pool은 sender를 holding
        ThreadPool { workers: workers, sender: Some(sender) }
    }

    /// closure를 thread에 할당 후 실행하는 method
    /// 
    /// `FnOnce` trait은 `F`가 반드시 한번만 호출될 수 있음을 의미함.
    /// `Send` trait은 closure를 한 thread에서 다른 thread로 보내야하기 때문에 필요.
    /// lifetime `'static`은 thread가 얼마나 오랫동안 실행될지 모르기 때문에 필요.
    pub fn execute<F>(&self, f: F) where F: FnOnce() + Send + 'static {
        let job = Box::new(f);
        
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    /// `TreadPool` instance가 drop될 때 안전하게 각 thread가 task를 끝낼 수 있도록 함
    fn drop(&mut self) {
        // sender drop함으로써 channel을 닫음
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            
            // join() method는 소유권을 요구하기 때문에 
            // Worker instance가 소유하고 있는 thread를 밖으로 빼낼 필요가 있음
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

/// thread 하나를 관리하는 worker
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// 빈 thread를 보유한 `Worker` instance를 반환
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            // mutex를 획득하기 위해 lock() method 호출
            // receiver가 channel로부터 Job을 얻기 위해 recv() method 호출
            let message = receiver.lock().unwrap().recv();
            
            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");
                    job();
                }
                // channel이 닫힐 경우 loop를 빠져 나감
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        Worker { id: id, thread: Some(thread) }
    }
}