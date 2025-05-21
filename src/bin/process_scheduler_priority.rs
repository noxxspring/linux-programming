use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq)]
enum Policy {
    CFS,
    FIFO,
    RR,
}

#[derive(Clone, Debug)]
struct Task {
    name: String,
    policy: Policy,
    priority: u8,
    vruntime: u32,
    time_slice: u32,
    total_runtime: u32,
}

struct Scheduler {
    rt_queue: VecDeque<Task>,
    cfs_queue: Vec<Task>,
    last_was_rt: bool, // track last scheduled task type
}

impl Scheduler {
    fn new() -> Self {
        Self {
            rt_queue: VecDeque::new(),
            cfs_queue: Vec::new(),
            last_was_rt: false,
        }
    }

    fn add_task(&mut self, task: Task) {
        match task.policy {
            Policy::CFS => self.cfs_queue.push(task),
            Policy::FIFO | Policy::RR => self.rt_queue.push_back(task),
        }
    }

    fn schedule(&mut self) -> Option<Task> {
        // Always run FIFO immediately if any present
        if let Some(pos) = self.rt_queue.iter().position(|t| t.policy == Policy::FIFO) {
            let mut task = self.rt_queue.remove(pos).unwrap();
            println!("→ [Real-Time FIFO] Running: {}", task.name);
            task.total_runtime += 10;
            self.last_was_rt = true; // RT task ran
            // FIFO is non-preemptive: no requeue
            return Some(task);
        }

        // Alternate between RT and CFS to avoid starvation
        if self.last_was_rt {
            // Last run was RT -> try running CFS if any
            if !self.cfs_queue.is_empty() {
                self.cfs_queue.sort_by_key(|t| t.vruntime);
                let mut task = self.cfs_queue.remove(0);
                println!("→ [CFS] Running: {}", task.name);
                task.vruntime += 10;
                task.total_runtime += 10;
                self.cfs_queue.push(task.clone());
                self.last_was_rt = false;
                return Some(task);
            } 
            // If no CFS tasks, fallback to RT
            if let Some(mut task) = self.rt_queue.pop_front() {
                println!("→ [Real-Time {:?}] Running: {}", task.policy, task.name);
                task.total_runtime += 10;
                if task.policy == Policy::RR {
                    // RR tasks requeue for round robin
                    self.rt_queue.push_back(task.clone());
                }
                self.last_was_rt = true;
                return Some(task);
            }
        } else {
            // Last run was CFS -> run RT if available
            if let Some(mut task) = self.rt_queue.pop_front() {
                println!("→ [Real-Time {:?}] Running: {}", task.policy, task.name);
                task.total_runtime += 10;
                if task.policy == Policy::RR {
                    self.rt_queue.push_back(task.clone());
                }
                self.last_was_rt = true;
                return Some(task);
            }
            // If no RT tasks, run CFS
            if !self.cfs_queue.is_empty() {
                self.cfs_queue.sort_by_key(|t| t.vruntime);
                let mut task = self.cfs_queue.remove(0);
                println!("→ [CFS] Running: {}", task.name);
                task.vruntime += 10;
                task.total_runtime += 10;
                self.cfs_queue.push(task.clone());
                self.last_was_rt = false;
                return Some(task);
            }
        }

        // No task to run
        None
    }
}

fn main() {
    let mut scheduler = Scheduler::new();

    scheduler.add_task(Task {
        name: "background-indexer".into(),
        policy: Policy::CFS,
        priority: 0,
        vruntime: 0,
        time_slice: 0,
        total_runtime: 0,
    });

    scheduler.add_task(Task {
        name: "Video-player".into(),
        policy: Policy::RR,
        priority: 1,
        vruntime: 0,
        time_slice: 10,
        total_runtime: 0,
    });

    scheduler.add_task(Task {
        name: "live-stream".into(),
        policy: Policy::FIFO,
        priority: 0,
        vruntime: 0,
        time_slice: 0,
        total_runtime: 0,
    });

    for _ in 0..15 {
        scheduler.schedule();
    }
}
